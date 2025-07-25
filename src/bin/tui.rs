use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::io;
use std::{time::Duration};

use dir_nuke::cli::{get_target_path, is_help};
use dir_nuke::cli::is_verbose;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::text::Span;
use ratatui::widgets::{Borders, Clear, ListState, Paragraph};
use rayon::prelude::*;
use walkdir::WalkDir;
use humansize::{format_size, DECIMAL};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    symbols::border,
    text::{Line},
    widgets::{Block, List, ListItem, Widget},
    DefaultTerminal, Frame,
};
// #[derive(Debug, Default)]
pub struct NodeModuleEntry {
    path: PathBuf,
    size_bytes: u64,
}

// #[derive(Debug, Default)]
enum AppState {
    ListDirs,
    Loading,
    ConfirmDelete,
    Exit,
}

// #[derive(Debug)]
pub struct App {
    state: AppState,
    spinner_index: usize,
    last_tick: Instant,
    list_state: ListState,
    selected: Vec<bool>,
    entries: Vec<NodeModuleEntry>,
}

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

impl App {

    pub fn new()-> App {
        let entries = get_dirs_on_path();
        let mut list_state = ListState::default();
        if !entries.is_empty() {
            list_state.select(Some(0));
        }
        let selected = vec![false; entries.len()];

        App {
            state: AppState::ListDirs,
            spinner_index: 0,
            last_tick: Instant::now(),
            list_state,
            selected,
            entries,
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let tick_rate = Duration::from_millis(10);
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            
            if event::poll(tick_rate)? {
                if let Event::Key(key) = event::read()? {
                    match self.state {
                        AppState::Exit => {
                            self.state = AppState::Exit;
                            break;
                        }
                        AppState::ListDirs => {
                            self.handle_events_on_list_dir(key)?;
                        },
                        AppState::Loading => match key.code {
                            KeyCode::Esc => self.state = AppState::ListDirs,
                            _ => {}
                        },
                        AppState::ConfirmDelete => match key.code {
                            KeyCode::Char('y') => {
                                // self.state = AppState::Loading; // simulate delete
                                // TODO: reload dirs
                                self.delete_selected();
                                self.reload_dirs();
                                self.state = AppState::Exit;
                            }
                            KeyCode::Char('n') | KeyCode::Esc => {
                                self.state = AppState::ListDirs;
                            }
                            _ => {}
                        },
                    }
                }
            }

            if matches!(self.state, AppState::Loading) {
                self.update_spinner();
            }


            
        }
        Ok(())
    }

    fn reload_dirs(&mut self) {
        self.state = AppState::Loading;
        self.spinner_index = 0;
        self.last_tick = Instant::now();

        let entries = get_dirs_on_path();
        let mut list_state = ListState::default();
        if !entries.is_empty() {
            list_state.select(Some(0));
        }
        let selected = vec![false; entries.len()];

        self.list_state = list_state;
        self.selected = selected;
        self.entries = entries;
        self.state = AppState::ListDirs;
    }

    fn draw(&self, frame: &mut Frame) {
        let frame_area = frame.area();
        // Main layout
        let block = Block::default().title("App").borders(Borders::ALL);
        frame.render_widget(block, frame_area);

        match self.state {
            AppState::Loading => {
                let spinner = SPINNER_FRAMES[self.spinner_index];
                let loading = Paragraph::new(format!("Loading... {}", spinner))
                    .style(Style::default().fg(Color::Yellow))
                    .alignment(Alignment::Center);
                let area = centered_rect(30, 10, frame_area);
                frame.render_widget(Clear, area); // clear area
                frame.render_widget(loading, area);
            }
            AppState::ConfirmDelete => {
                let area = centered_rect(60, 30, frame_area);
                let question = Paragraph::new(vec![
                    Line::from(Span::raw("Are you sure you want to delete?")),
                    Line::from(Span::raw("")),
                    Line::from(Span::styled("Confirm (Y) | Cancel (N)", Style::default().fg(Color::Yellow))),
                ])
                .alignment(Alignment::Center)
                .block(Block::default().title("Confirm Delete").borders(Borders::ALL));
                frame.render_widget(Clear, area); // clear area
                frame.render_widget(question, area);
            }
            _ => {
                // let help = Paragraph::new("Press 'd' to delete, 'l' to load, 'q' to quit.")
                //     .alignment(Alignment::Center);
                // frame.render_widget(help, centered_rect(50, 5, frame_area));
                frame.render_widget(self, frame_area);
            }
        }
    }

    // updates the application's state based on user input
    fn handle_events_on_list_dir(&mut self, key_event: KeyEvent) -> io::Result<()> {
        // it's important to check that the event is a key press event as
        // crossterm also emits key release and repeat events on Windows.
        match key_event.kind {
            KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Esc | KeyCode::Char('q') => self.state = AppState::Exit,
                    KeyCode::Enter => {
                        self.state = AppState::ConfirmDelete;
                    },
                    KeyCode::Char('l') => self.select_item(),
                    KeyCode::Char('h') => self.unselect_item(),
                    KeyCode::Char(' ') => self.toggle_item_selection(),
                    KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => self.move_up(),
                    KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => self.move_down(),
                    _ => {}
                }
            },
            _ => {}
        };
        Ok(())
    }

    fn delete_selected(&self) {
        let to_delete: Vec<&NodeModuleEntry> = self.entries
            .iter()
            .zip(self.selected.iter())
            .filter(|&(_, sel)| *sel)
            .map(|(entry, _)| entry)
            .collect();

        // self.messages.push(format!("🗑 Deleting {} folders...", to_delete.len()));
        for entry in to_delete {
            if let Err(_) = fs::remove_dir_all(&entry.path) {
                // self.messages.push(format!("❌ Failed to delete {}: {}", entry.path.display(), e));
            } else {
                // TODO: fix messaging. Print outside when close.
                // Maybe post on app_result
                // self.messages.push(format!("✅ Deleted {}", entry.path.display()));
                // thread::sleep(Duration::from_millis(300));
                // self.exit();
            }
        }

        // self.messages.push("🎉 Done.".to_string());
    }

    fn select_item(&mut self) {
        let i = self.list_state.selected().unwrap_or(0);
        self.selected[i] = true;
    }

    fn unselect_item(&mut self) {
        let i = self.list_state.selected().unwrap_or(0);
        self.selected[i] = false;
    }

    fn toggle_item_selection(&mut self) {
        let i = self.list_state.selected().unwrap_or(0);
        self.selected[i] = !self.selected[i];
    }

    fn move_up(&mut self) {
        let i = self.list_state.selected().unwrap_or(0);
        let prev = if i == 0 { self.entries.len() - 1 } else { i - 1 };
        self.list_state.select(Some(prev));
    }

    fn move_down(&mut self) {
        let i = self.list_state.selected().unwrap_or(0);
        let next = if i >= self.entries.len() - 1 { 0 } else { i + 1 };
        self.list_state.select(Some(next));
    }

    fn update_spinner(&mut self) {
        if self.last_tick.elapsed() >= Duration::from_millis(100) {
            self.spinner_index = (self.spinner_index + 1) % SPINNER_FRAMES.len();
            self.last_tick = Instant::now();
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use ratatui::layout::{Constraint, Direction, Layout};

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0), // For the list
            ])
            .split(area);

        let title = Line::from(" dir-nuke 💥".bold());
        let instructions = Line::from(vec![
            " Toggle selection ".into(),
            "<Space>".blue().bold(),
            " | ".into(),
            " Delete selected ".into(),
            "<Enter>".blue().bold(),
            " | ".into(),
            " Quit ".into(),
            "<Esc> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let items: Vec<ListItem> = self.entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let prefix = if self.selected[i] { "[x] " } else { "[ ] " };
                ListItem::new(prefix.to_string() + &human_label(entry))
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

        // Render the list using the list_state
        ratatui::widgets::StatefulWidget::render(list, chunks[0], buf, &mut self.list_state.clone());

    }
}

fn find_node_modules(base: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let mut walker = WalkDir::new(base).into_iter();

    while let Some(entry_result) = walker.next() {
        match entry_result {
            Ok(entry) => {
                if entry.file_type().is_dir() && entry.file_name() == "node_modules" {
                    result.push(entry.path().to_path_buf());
                    walker.skip_current_dir(); // ✅ safe now
                }
            }
            Err(_) => continue, // Ignore errors
        }
    }

    result
}

fn dir_size_recursive(path: &Path) -> u64 {
    let mut size = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(metadata) = fs::symlink_metadata(&path) {
                if metadata.is_file() {
                    size += metadata.len();
                } else if metadata.is_dir() {
                    size += dir_size_recursive(&path);
                }
            }
        }
    }
    size
}

fn calculate_sizes(dirs: &[PathBuf]) -> Vec<NodeModuleEntry> {
    dirs.par_iter()
        .map(|path| NodeModuleEntry {
            path: path.clone(),
            size_bytes: dir_size_recursive(path),
        })
        .collect()
}

fn human_label(entry: &NodeModuleEntry) -> String {
    let size = format_size(entry.size_bytes, DECIMAL);
    format!("{} - {}", size, entry.path.display())
}

// Utility to center a widget
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn get_dirs_on_path() -> Vec<NodeModuleEntry> {
    let target_dir = get_target_path();
    let base_path = Path::new(&target_dir);

    println!("🔍 Scanning for node_modules folders in {:?}...", base_path);
    let scan_start = Instant::now();
    let found_dirs = find_node_modules(base_path);
    if found_dirs.is_empty() {
        println!("✅ No node_modules folders found.");
        return vec![];
    }
    let search_duration = scan_start.elapsed();
    if is_verbose(){
        println!("⏰ Scan duration was: {:?}", search_duration);
    }

    println!("📦 Calculating sizes...");
    let mut entries = calculate_sizes(&found_dirs);
    entries.sort_by_key(|e| std::cmp::Reverse(e.size_bytes));
    return entries;
}

fn main() -> io::Result<()>{
    if is_help() {
        println!("dir-nuke is a safe and fast CLI tool to delete or \"nuke\" directories.\n        Usage: dir-nuke <search_path>        \n");
        return Ok(());
    }
    
    // TODO: calculate sum of size_bytes in entries

    // -- NEW tui
    let mut terminal = ratatui::init();
    let mut app = App::new();
    let app_result = app.run(&mut terminal);
    ratatui::restore();
    app_result

}
