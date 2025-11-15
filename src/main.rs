mod app;
mod cli;
mod utils;

use app::App;
use clap::Parser;
use cli::Cli;
use color_eyre::{Result, eyre::Ok};

use std::fs::read_dir;
use std::process::exit;

fn desktop_exists(name: Option<String>) -> bool {
    let trimmed = name.unwrap();
    let file_name = format!("{}.desktop", trimmed);

    if trimmed.is_empty() {
        return false;
    }

    let path = dirs::data_dir()
        .unwrap()
        .join("applications")
        .join(&file_name);

    path.exists()
}

fn list_all_desktop_files() {
    let mut counter: usize = 0;
    let dir = dirs::data_dir().unwrap().join("applications");

    if !dir.exists() {
        eprintln!("[ERROR]: No applications directory found!");
        return;
    }

    let entries = match read_dir(&dir) {
        std::result::Result::Ok(e) => e,
        Err(err) => {
            eprintln!("[ERROR]: Failed to read directory: {}", err);
            return;
        }
    };

    println!("[DESKFORGE]");
    for entry in entries {
        if let std::result::Result::Ok(entry) = entry {
            let path = entry.path();

            if let Some(ext) = path.extension() {
                if ext == "desktop" {
                    if let Some(name) = path.file_name() {
                        counter += 1;
                        println!("{}. {}", counter, name.to_string_lossy());
                    }
                }
            }
        }
    }
    println!("Total: {}", counter);
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    if cli.list {
        list_all_desktop_files();
        return Ok(());
    }

    if let Some(name) = cli.new {
        let file_name = name;

        if desktop_exists(file_name.clone()) {
            eprintln!("[ERROR]: File name already exist!");
            exit(1);
        }

        let mut terminal = ratatui::init();
        let result = App::new(Some(file_name.unwrap_or_default()), false).run(&mut terminal);
        ratatui::restore();
        return result;
    }

    if let Some(name) = cli.edit {
        let trimmed = name.trim();

        let file_name = if trimmed.ends_with(".desktop") {
            trimmed.to_string()
        } else {
            format!("{}.desktop", trimmed)
        };

        let path = dirs::data_dir()
            .unwrap()
            .join("applications")
            .join(&file_name);

        if !path.exists() {
            eprintln!("[ERROR]: File doesn't exists!");
            exit(1)
        }

        let mut terminal = ratatui::init();
        let result = App::new(Some(file_name), true).run(&mut terminal);
        ratatui::restore();
        return result;
    } else {
        eprintln!("[WARNING]: Please enter a file name!");
        exit(1);
    }
}
