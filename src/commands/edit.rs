use crate::App;
use color_eyre::Result;

pub fn edit_err(file_name: &str) -> bool {
    let path = dirs::data_dir()
        .unwrap()
        .join("applications")
        .join(file_name);

    if !path.exists() {
        return true;
    }

    return false;
}

pub fn edit(file_name: String) -> Result<()> {
    let mut terminal = ratatui::init();
    let result = App::new(Some(file_name), true).run(&mut terminal);
    ratatui::restore();
    return result;
}
