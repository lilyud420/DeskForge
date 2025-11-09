use clap::{Parser};

#[derive(Parser)]
#[command(name = "DeskForge")]
#[command(version = ": 1.0")]
pub struct Cli {
    #[arg(short = 'n', long = "new", help = "Create a new launcher")]
    pub new: bool,
    #[arg(short = 'e', long = "edit", help = "Edit an exisiting launcher")]
    pub edit: Option<String>,
}
