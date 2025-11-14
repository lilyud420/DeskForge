mod app;
mod cli;
mod utils;

use std::process::exit;

use app::App;
use clap::Parser;
use cli::Cli;
use color_eyre::Result;

fn desktop_exists(name: Option<String>) -> bool {
    let trimmed = name.unwrap();
    let file_name = format!("{}.desktop", trimmed);

    if trimmed.is_empty(){
        return false;
    }
    
    let path = dirs::data_dir()
        .unwrap()
        .join("applications")
        .join(&file_name);
    
    path.exists()
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    if let Some(name) = cli.new {
        let file_name = name;
        
        if desktop_exists(file_name.clone()) {
            eprintln!("File name already exist!");
            exit(1);
        }
        
        let mut terminal = ratatui::init();
        let result = App::new(Some(file_name.unwrap_or_default())).run(&mut terminal);
        ratatui::restore();
        return result;
    }

    if let Some(file_name) = cli.edit {
        let mut terminal = ratatui::init();
        let result = App::new(Some(file_name)).run(&mut terminal);
        ratatui::restore();
        return result;
    } else {
        Ok(())
    }
}
