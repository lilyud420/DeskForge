mod app;
mod cli;
mod commands;
mod utils;

use crate::commands::remove::{remove, remove_err};
use crate::{commands::edit::*, commands::list::list_all_desktop_files, commands::new::*};

use app::App;
use clap::{CommandFactory, Parser};
use cli::Cli;
use color_eyre::{Result, eyre::Ok};
use dirs::data_dir;

use std::fs::create_dir_all;
use std::path::PathBuf;
use std::process::exit;

fn applications_dir() -> PathBuf {
    let data_dir = match data_dir() {
        Some(d) => d,
        None => {
            eprintln!("[ERROR]: Unexpected error");
            exit(1);
        }
    };

    if let Err(e) = create_dir_all(&data_dir) {
        eprintln!("[ERROR]: Cannot create data_dir: {e}");
        exit(1);
    }

    let app_dir = data_dir.join("applications");

    if let Err(e) = create_dir_all(&app_dir) {
        eprintln!("[ERROR]: Cannot create app_dir: {e}");
        exit(1);
    }

    app_dir
}

fn normalize_desktop_name(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.ends_with(".desktop") {
        trimmed.to_string()
    } else {
        format!("{trimmed}.desktop")
    }
}

fn desktop_exists(name: &str) -> bool {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return false;
    }
    let file_name = normalize_desktop_name(trimmed);
    let file_path = applications_dir().join(&file_name);
    file_path.exists()
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    if cli.list {
        list_all_desktop_files();
        return Ok(());
    }

    match cli.remove {
        None => {}
        Some(name) => {
            let file_name = normalize_desktop_name(&name);

            if remove_err(&file_name) {
                eprintln!("[ERROR]: File doesn't exist!");
                exit(1)
            }

            remove(&file_name);
            return Ok(());
        }
    }

    match cli.new {
        None => {}
        Some(None) => {
            return new_default_file();
        }
        Some(Some(name)) => {
            if desktop_exists(&name) {
                eprintln!("[ERROR]: File name already exists!");
                exit(1);
            }
            return new_file(Some(name));
        }
    }

    if let Some(name) = cli.edit {
        let file_name = normalize_desktop_name(&name);

        if edit_err(&file_name) {
            eprintln!("[ERROR]: File doesn't exist!");
            exit(1)
        }

        return edit(file_name);
    }
    eprintln!("[WARNING]: Wrong command!");
    Cli::command().print_help().unwrap();
    exit(1);
}
