use dialoguer::{theme::ColorfulTheme, MultiSelect};
use duct::cmd;
use std::fs;
use std::process;

fn main() {
    let target_dir = std::env::args().nth(1).unwrap_or_else(|| ".".into());

    println!("🔍 Searching for node_modules in {}", target_dir);

    let find_cmd = format!(
        "find {} -type d -name node_modules -prune -exec du -sh {{}} +",
        target_dir
    );

    let output = cmd!("sh", "-c", &find_cmd)
        .read()
        .expect("Failed to run find/du");

    let lines: Vec<_> = output
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() == 2 {
                Some((parts[0].to_string(), parts[1].to_string()))
            } else {
                None
            }
        })
        .collect();

    if lines.is_empty() {
        println!("✅ No node_modules found.");
        return;
    }

    let items: Vec<String> = lines
        .iter()
        .map(|(size, path)| format!("{} - {}", size, path))
        .collect();

    let selection = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select node_modules folders to delete")
        .items(&items)
        .interact()
        .unwrap();

    if selection.is_empty() {
        println!("❌ Nothing selected.");
        return;
    }

    println!("⚠️ You selected {} directories to delete.", selection.len());

    for index in selection {
        let (_, path) = &lines[index];
        println!("🗑 Deleting {}", path);
        if let Err(e) = fs::remove_dir_all(path) {
            eprintln!("❌ Failed to delete {}: {}", path, e);
        }
    }

    println!("✅ Done.");
}
