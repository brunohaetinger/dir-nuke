use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::io;

use dir_nuke::cli::get_target_path;
use dir_nuke::cli::is_verbose;
use ratatui::widgets::{ListState, Paragraph};
use rayon::prelude::*;
use walkdir::WalkDir;
use humansize::{format_size, DECIMAL};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, List, ListItem, Widget},
    DefaultTerminal, Frame,
};
#[derive(Debug, Default)]
pub struct NodeModuleEntry {
    path: PathBuf,
    size_bytes: u64,
}

#[derive(Debug, Default)]
pub struct App {
    list_state: ListState,
    selected: Vec<bool>,
    entries: Vec<NodeModuleEntry>,
    exit: bool,
    messages: Vec<String>, // New field to store messages
}


impl App {

    pub fn new(entries: Vec<NodeModuleEntry>)-> App {
        let mut list_state = ListState::default();
        if !entries.is_empty() {
            list_state.select(Some(0));
        }
        let selected = vec![false; entries.len()];
        let messages = Vec::new(); // Initialize messages

        App {
            list_state,
            selected,
            entries,
            exit: false,
            messages, // Add messages to the struct
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    // updates the application's state based on user input
    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.exit(),
            KeyCode::Delete => self.delete_selected(),
            KeyCode::Char('l') => self.select_item(),
            KeyCode::Char('h') => self.unselect_item(),
            KeyCode::Char(' ') => self.toggle_item_selection(),
            KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => self.move_up(),
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => self.move_down(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn delete_selected(&mut self) {
        let to_delete: Vec<&NodeModuleEntry> = self.entries
            .iter()
            .zip(self.selected.iter())
            .filter(|&(_, sel)| *sel)
            .map(|(entry, _)| entry)
            .collect();

        self.messages.push(format!("🗑 Deleting {} folders...", to_delete.len()));
        for entry in to_delete {
            if let Err(e) = fs::remove_dir_all(&entry.path) {
                self.messages.push(format!("❌ Failed to delete {}: {}", entry.path.display(), e));
            } else {
                self.messages.push(format!("✅ Deleted {}", entry.path.display()));
                self.exit = true;
            }
        }

        self.messages.push("🎉 Done.".to_string());
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
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use ratatui::layout::{Constraint, Direction, Layout};

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0), // For the list
                Constraint::Length(self.messages.len() as u16 + 2), // For messages + border
            ])
            .split(area);

        let title = Line::from(" dir-nuke 💥".bold());
        let instructions = Line::from(vec![
            " Toggle selection ".into(),
            "<Space>".blue().bold(),
            " | ".into(),
            " Delete selected ".into(),
            "<Del>".blue().bold(),
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

        // Render messages
        let messages_text: Vec<Line> = self.messages.iter().map(|msg| Line::from(msg.clone())).collect();
        let messages_block = Block::bordered().title("Messages");
        let messages_paragraph = Paragraph::new(Text::from(messages_text)).block(messages_block);
        messages_paragraph.render(chunks[1], buf);
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

fn main() -> io::Result<()>{
    let target_dir = get_target_path();
    let base_path = Path::new(&target_dir);

    println!("🔍 Scanning for node_modules folders in {:?}...", base_path);
    let scan_start = Instant::now();
    let found_dirs = find_node_modules(base_path);
    if found_dirs.is_empty() {
        println!("✅ No node_modules folders found.");
        return Ok(());
    }
    let search_duration = scan_start.elapsed();
    if is_verbose(){
        println!("⏰ Scan duration was: {:?}", search_duration);
    }

    println!("📦 Calculating sizes...");
    let mut entries = calculate_sizes(&found_dirs);
    entries.sort_by_key(|e| std::cmp::Reverse(e.size_bytes));

    // TODO: calculate sum of size_bytes in entries

    // -- NEW tui
    let mut terminal = ratatui::init();
    let mut app = App::new(entries);
    let app_result = app.run(&mut terminal);
    ratatui::restore();
    app_result

}
