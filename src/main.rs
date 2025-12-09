mod app;
mod cli;
mod commands;
mod utils;

use crate::{commands::edit::*, commands::list::list_all_desktop_files, commands::new::*};

use app::App;
use clap::{CommandFactory, Parser};
use cli::Cli;
use color_eyre::{Result, eyre::Ok};
use dirs::data_dir;

use std::fs::create_dir_all;
use std::process::exit;

fn desktop_exists(name: Option<String>) -> bool {
    let trimmed = name.unwrap();
    let file_name = format!("{}.desktop", trimmed);

    if trimmed.is_empty() {
        return false;
    }

    let data_dir = match data_dir() {
        Some(d) => d,
        None => {
            eprintln!("[ERROR]: Unexpected error");
            exit(1);
        }
    };

    if let Err(e) = create_dir_all(&data_dir) {
        eprintln!("[ERROR]: Cannot create data_dir {e}");
        exit(1);
    }

    let app_dir = data_dir.join("applications");

    if let Err(e) = create_dir_all(&app_dir) {
        eprintln!("[ERROR]: Cannot create app_dir {e}");
        exit(1);
    }

    let file_path = app_dir.join(&file_name);

    file_path.exists()
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    if cli.list {
        list_all_desktop_files();
        return Ok(());
    }

    match cli.new {
        None => {}
        Some(None) => {
            return new_default_file();
        }
        Some(Some(name)) => {
            if desktop_exists(Some(name.clone())) {
                eprintln!("[ERROR]: File name already exists!");
                exit(1);
            }
            return new_file(Some(name));
        }
    }

    if let Some(name) = cli.edit {
        let trimmed = name.trim();

        let file_name = if trimmed.ends_with(".desktop") {
            trimmed.to_string()
        } else {
            format!("{}.desktop", trimmed)
        };

        if edit_err(&file_name) {
            eprintln!("[ERROR]: File doesn't exists!");
            exit(1)
        }

        return edit(file_name);
    } else {
        eprintln!("[WARNING]: Wrong command!");
        Cli::command().print_help().unwrap();
        exit(1);
    }
}
