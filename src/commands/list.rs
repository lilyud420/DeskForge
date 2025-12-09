use std::fs::read_dir;

pub fn list_all_desktop_files() {
    let mut counter: usize = 0;
    let dir = dirs::data_dir().unwrap().join("applications");

    if !dir.exists() {
        eprintln!("[ERROR]: No applications directory found!");
        return;
    }

    let entries = match read_dir(&dir) {
        std::result::Result::Ok(e) => e,
        Err(err) => {
            eprintln!("[ERROR]: Failed to read directory: {}", err);
            return;
        }
    };

    println!("[DESKFORGE]");
    for entry in entries {
        if let std::result::Result::Ok(entry) = entry {
            let path = entry.path();

            if let Some(ext) = path.extension() {
                if ext == "desktop" {
                    if let Some(name) = path.file_name() {
                        counter += 1;
                        println!("{}. {}", counter, name.to_string_lossy());
                    }
                }
            }
        }
    }
    println!("Total: {}", counter);
}
