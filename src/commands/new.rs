use crate::App;
use color_eyre::Result;

pub fn new_default_file() -> Result<()> {
    let default_name = "".to_string();

    let mut terminal = ratatui::init();
    let result = App::new(Some(default_name), false).run(&mut terminal);
    ratatui::restore();
    return result;
}

pub fn new_file(name: Option<String>) -> Result<()> {
    let mut terminal = ratatui::init();
    let result = App::new(name, false).run(&mut terminal);
    ratatui::restore();
    return result;
}
