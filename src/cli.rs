use clap::{Parser};

#[derive(Parser)]
#[command(name = "DeskForge", version = "1.0", about = "A launcher creation tool", override_usage = "deskforge [COMMANDS] [OPTIONS]")]
pub struct Cli {
    /// Create a new launcher
    #[arg(short = 'n', long = "new", value_name = "FILE_NAME", num_args = 0..=1)]
    pub new: Option<String>,
    
    /// Edit an existing launcher
    #[arg(short = 'e', long = "edit", value_name = "FILE_NAME", num_args= 0..=1)]
    pub edit: Option<String>,
}
