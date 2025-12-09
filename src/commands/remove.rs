use std::fs::remove_file;

pub fn remove_err(file_name: &str) -> bool {
    let path = dirs::data_dir()
        .unwrap()
        .join("applications")
        .join(file_name);

    if !path.exists() {
        return true;
    }

    return false;
}
pub fn remove(file_name: &str) {
    let path = dirs::data_dir()
        .unwrap()
        .join("applications")
        .join(file_name);

    remove_file(&path).unwrap();
}
