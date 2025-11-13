mod cli;
mod app;
mod utils;

use color_eyre::Result;
use clap::Parser;
use cli::Cli;
use app::App;

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    if let Some(name) = cli.new {
        let file_name = name.unwrap_or_default();
        let mut terminal = ratatui::init();
        let result = App::new(Some(file_name)).run(&mut terminal);
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
