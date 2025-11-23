use clap::Parser;

#[derive(Parser)]
#[command(
    name = "DeskForge",
    version = "1.0",
    about = "Deskforge - A launcher creation tool",
    override_usage = "deskforge [COMMANDS] [OPTIONS]"
)]
pub struct Cli {
    /// Create a new launcher
    #[arg(short = 'n', long = "new", value_name = "OPTIONAL: FILE_NAME", num_args = 0..=1)]
    pub new: Option<Option<String>>,

    /// Edit an existing launcher
    #[arg(short = 'e', long = "edit", value_name = "REQUIRED: FILE_NAME", num_args= 0..=1)]
    pub edit: Option<String>,

    /// List all exisiting launcher
    #[arg(short = 'l', long = "list")]
    pub list: bool,
}
