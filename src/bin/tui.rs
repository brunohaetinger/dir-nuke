use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use walkdir::WalkDir;
use humansize::{format_size, DECIMAL};

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Terminal;
use ratatui::layout::{Layout, Constraint, Direction};
use ratatui::style::{Style, Modifier, Color};

struct NodeModuleEntry {
    path: PathBuf,
    size_bytes: u64,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target = env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let base_path = Path::new(&target);

    println!("🔍 Scanning for node_modules folders in {:?}...", base_path);
    let found_dirs = find_node_modules(base_path);
    if found_dirs.is_empty() {
        println!("✅ No node_modules folders found.");
        return Ok(());
    }

    println!("📦 Calculating sizes...");
    let mut entries = calculate_sizes(&found_dirs);
    entries.sort_by_key(|e| std::cmp::Reverse(e.size_bytes));

    // Setup terminal UI
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let items: Vec<ListItem> = entries
        .iter()
        .map(|e| ListItem::new(human_label(e)))
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(0));
    let mut selected = vec![false; entries.len()];

    terminal.clear().unwrap();
    loop {
        terminal.draw(|f| {
            let size = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
                .split(size);

            let items_rendered: Vec<ListItem> = items
                .iter()
                .enumerate()
                .map(|(i, _item)| {
                    let prefix = if selected[i] { "[x] " } else { "[ ] " };
                    ListItem::new(prefix.to_string() + &human_label(&entries[i]))
                })
                .collect();

            let list = List::new(items_rendered)
                .block(Block::default().borders(Borders::ALL).title("Select node_modules to delete (space = toggle ✔️, enter = confirm ✅)"))
                .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

            f.render_stateful_widget(list, chunks[0], &mut list_state);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
                        let i = list_state.selected().unwrap_or(0);
                        let next = if i >= entries.len() - 1 { 0 } else { i + 1 };
                        list_state.select(Some(next));
                    }
                    KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab  => {
                        let i = list_state.selected().unwrap_or(0);
                        let prev = if i == 0 { entries.len() - 1 } else { i - 1 };
                        list_state.select(Some(prev));
                    }
                    KeyCode::Char(' ') => {
                        let i = list_state.selected().unwrap_or(0);
                        selected[i] = !selected[i];
                    }
                    KeyCode::Char('h') => {
                        let i = list_state.selected().unwrap_or(0);
                        selected[i] = false;
                    }
                    KeyCode::Char('l') => {
                        let i = list_state.selected().unwrap_or(0);
                        selected[i] = true;
                    }
                    KeyCode::Enter => {
                        break;
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        disable_raw_mode()?;
                        crossterm::execute!(terminal.backend_mut(), DisableMouseCapture)?;
                        terminal.show_cursor()?;
                        terminal.clear()?;
                        println!("\n❌ Cancelled.");
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), DisableMouseCapture)?;
    terminal.show_cursor()?;

    let to_delete: Vec<&NodeModuleEntry> = entries
        .iter()
        .zip(selected.iter())
        .filter(|&(_, sel)| *sel)
        .map(|(entry, _)| entry)
        .collect();

    println!("🗑 Deleting {} folders...", to_delete.len());
    for entry in to_delete {
        if let Err(e) = fs::remove_dir_all(&entry.path) {
            eprintln!("❌ Failed to delete {}: {}", entry.path.display(), e);
        } else {
            println!("✅ Deleted {}", entry.path.display());
        }
    }

    println!("🎉 Done.");
    Ok(())
}
