use crate::utils::constants::*;

use color_eyre::eyre::Ok;
use color_eyre::eyre::Result;
use is_executable::IsExecutable;
use ratatui::{
    crossterm::event::KeyCode,
    style::{Color, Style},
};
use tui_input::Input;

use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::{
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct App {
    pub input_mode: InputMode,
    pub input: Vec<Input>,
    pub last_key: Option<KeyCode>,

    pub dropdown_open: bool,
    pub dropdown_options: Vec<&'static str>,
    pub dropdown_selected: usize,
    pub dropdown_index: Option<usize>,

    pub block_index: usize,

    pub checkbox_nodisplay: bool,
    pub checkbox_startupnotify: bool,
    pub checkbox_terminal: bool,

    pub edit: bool,
    pub exit: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Insert,
}

impl App {
    pub fn new(file_name: Option<String>, file_edit: bool) -> Self {
        let mut input = vec![Input::default(); NUM_BLOCK];
        let mut block_index: usize = 0;
        let mut edit = false;

        input[IDX_TYPE] = Input::from("Application");
        input[IDX_CATEGORY] = Input::from("None");

        if let Some(name) = file_name.clone()
            && name != ""
        {
            input[0] = Input::from(name);
            block_index += 1;
        }

        if file_edit {
            edit = true;
            if let Some(name) = &file_name {
                let path: PathBuf = dirs::data_dir()
                    .unwrap_or_else(|| PathBuf::from("/tmp"))
                    .join("applications")
                    .join(name);

                if path.exists() {
                    if let std::result::Result::Ok(file) = File::open(&path) {
                        let reader = BufReader::new(file);

                        for line in reader.lines().flatten() {
                            if let Some((key, value)) = line.split_once('=') {
                                match key {
                                    "Name" => input[IDX_NAME] = Input::from(value),
                                    "Exec" => input[IDX_EXEC] = Input::from(value),
                                    "URL" => input[IDX_URL] = Input::from(value),
                                    "Icon" => input[IDX_ICON] = Input::from(value),
                                    "Version" => input[IDX_VERSION] = Input::from(value),
                                    "Comment" => input[IDX_COMMENT] = Input::from(value),
                                    "Actions" => input[IDX_ACTION] = Input::from(value),
                                    "NoDisplay" => input[IDX_NODISPLAY] = Input::from(value),
                                    "StartupNotify" => {
                                        input[IDX_STARTUPNOTIFY] = Input::from(value)
                                    }
                                    "Terminal" => input[IDX_TERMINAL] = Input::from(value),
                                    "Type" => input[IDX_TYPE] = Input::from(value),
                                    "Category" => input[IDX_CATEGORY] = Input::from(value),

                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }

        Self {
            block_index,
            input_mode: InputMode::Normal,
            input,

            dropdown_open: false,
            dropdown_options: Vec::new(),
            dropdown_selected: 0,
            dropdown_index: None,

            last_key: None,

            checkbox_nodisplay: false,
            checkbox_startupnotify: true,
            checkbox_terminal: false,

            edit,
            exit: false,
        }
    }

    pub fn next_block(&mut self) {
        if self.block_index == IDX_CANCEL {
            return;
        }
        self.block_index += 1;
    }

    pub fn previous_block(&mut self) {
        if self.block_index == IDX_NAME {
            return;
        }
        self.block_index -= 1;
    }

    pub fn submit_message(&mut self) {
        self.next_block();
    }

    pub fn open_dropdown(&mut self, index: usize, options: Vec<&'static str>) {
        self.dropdown_open = true;
        self.dropdown_options = options;
        self.dropdown_selected = 0;
        self.dropdown_index = Some(index);
        self.input[index] = Input::from(self.dropdown_options[0]);
    }

    pub fn save_as_desktop(&self, file_name: &str) -> Result<()> {
        let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("/tmp"));

        path.push("applications");
        path.push(file_name);

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&path)?;

        writeln!(file, "[Desktop Entry]")?;
        writeln!(file, "Name={}", self.input[IDX_NAME])?;

        match self.input[IDX_TYPE].value() {
            "Link" => writeln!(file, "URL={}", self.input[IDX_URL])?,
            "Application" => writeln!(file, "Exec={}", self.input[IDX_EXEC])?,
            "Directory" => writeln!(file, "Exec={}", self.input[IDX_EXEC])?,
            "Application (other)" => writeln!(file, "Exec={}", self.input[IDX_EXEC])?,
            _ => {}
        }

        writeln!(file, "Icon={}", self.input[IDX_ICON])?;
        writeln!(file, "Version={}", self.input[IDX_VERSION])?;
        writeln!(file, "Comment={}", self.input[IDX_COMMENT])?;
        writeln!(file, "Actions={}", self.input[IDX_ACTION])?;
        writeln!(
            file,
            "NoDisplay={}",
            if self.checkbox_nodisplay {
                "true"
            } else {
                "false"
            }
        )?;
        writeln!(
            file,
            "StartupNotify={}",
            if self.checkbox_nodisplay {
                "true"
            } else {
                "false"
            }
        )?;
        writeln!(
            file,
            "Terminal={}",
            if self.checkbox_terminal {
                "true"
            } else {
                "false"
            }
        )?;
        writeln!(file, "Type={}", self.input[IDX_TYPE])?;
        writeln!(
            file,
            "Category={}",
            if self.input[IDX_CATEGORY].value().eq("None") {
                ""
            } else {
                self.input[IDX_CATEGORY].value()
            }
        )?;
        Ok(())
    }

    pub fn can_save(&self) -> bool {
        let name = self.input[IDX_NAME].value().trim();
        if name.is_empty() {
            return false;
        }

        let file_name = format!("{}.desktop", name);

        let save_path = dirs::data_dir()
            .unwrap()
            .join("applications")
            .join(&file_name);

        if !self.edit {
            if save_path.exists() {
                return false;
            }
        } else {
            true;
        }

        match self.input[IDX_TYPE].value() {
            "Link" => {
                let url = self.input[IDX_URL].value().trim();

                if url.is_empty() {
                    return false;
                }

                if url.starts_with("file://") {
                    if let Some(local_path) = url.strip_prefix("file://") {
                        let path = Path::new(local_path);
                        return path.exists();
                    }
                }

                url.starts_with("https://")
                    || url.starts_with("http://")
                    || url.starts_with("mailto:")
                    || url.starts_with("smb://")
                    || url.starts_with("trash:///")
                    || url.starts_with("recent:///")
            }
            "Application" => {
                let trimmed = self.input[IDX_EXEC].value().trim();
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                let special_path = parts.iter().any(|p| p.contains("//"));

                if special_path {
                   return true; 
                }

                if let Some(exec_raw) = parts.iter().find(|&&p| p.contains("/")) {
                    let exec_path = Path::new(exec_raw);

                    exec_path.exists()
                } else {
                    false
                }
            }
            _ => true,
        }
    }

    pub fn validate_path(&self, input: &str, exts: &[&str], index: usize) -> (Style, String) {
        let trimmed = input.trim();
        let path = Path::new(trimmed);

        if self.block_index != index {
            return (Style::default(), "".to_string());
        }

        if !path.exists() {
            match self.input[IDX_TYPE].value() {
                "Application (other)" | "Directory" => {
                    return (
                        self.is_active_block_style(IDX_EXEC),
                        "- Ignored".to_string(),
                    );
                }
                _ => {}
            }
        }

        if trimmed.is_empty() {
            match index {
                IDX_EXEC => {
                    return (Style::default().fg(Color::LightRed), " - Empty".to_string());
                }
                IDX_ICON => {
                    return (Style::default().fg(Color::LightRed), " - Empty".to_string());
                }
                _ => {}
            }
        }

        if self.input[IDX_TYPE].value().eq("Link") {
            if trimmed.is_empty() {
                return (Style::default().fg(Color::LightRed), " - Empty".to_string());
            }

            if !(trimmed.starts_with("file://")
                || trimmed.starts_with("https://")
                || trimmed.starts_with("http://")
                || trimmed.starts_with("mailto:")
                || trimmed.starts_with("smb://")
                || trimmed.starts_with("trash:///")
                || trimmed.starts_with("recent:///"))
            {
                return (
                    Style::default().fg(Color::Yellow),
                    " - Invalid scheme".to_string(),
                );
            }

            if trimmed.starts_with("file://") {
                if let Some(local_path) = trimmed.strip_prefix("file://") {
                    let path = Path::new(local_path);
                    if !path.exists() {
                        return (
                            Style::default().fg(Color::LightRed),
                            "- Not found".to_string(),
                        );
                    }
                }
            }

            return (Style::default().fg(Color::LightGreen), "- OK".to_string());
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let exec_path_opt = parts.iter().find(|&&p| p.contains('/'));

        match exts.is_empty() {
            true => {
                if let Some(exec_path) = exec_path_opt {
                    let path = Path::new(exec_path);

                    if !path.exists() {
                        return (
                            Style::default().fg(Color::LightRed),
                            " - Not found".to_string(),
                        );
                    }

                    if !path.is_executable() {
                        return (
                            Style::default().fg(Color::Yellow),
                            " - Unexpected type".to_string(),
                        );
                    }

                    return (Style::default().fg(Color::LightGreen), " - OK".to_string());
                }
            }
            false => {}
        }

        if !exts.is_empty() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if exts.iter().any(|&v| v.eq_ignore_ascii_case(ext)) {
                    return (Style::default().fg(Color::LightGreen), " - OK".to_string());
                }
            }
            return (
                Style::default().fg(Color::Yellow),
                " - Unexpected type".to_string(),
            );
        }

        return (Style::default().fg(Color::LightGreen), " - OK".to_string());
    }

    pub fn validate_name(&self, input: &str, index: usize) -> (Style, String) {
        let trimmed = input.trim();

        if self.block_index != index {
            return (Style::default(), "".to_string());
        }

        if self.edit {
            return (
                Style::default().fg(Color::LightGreen),
                " - Ignored".to_string(),
            );
        }

        if trimmed.is_empty() {
            return (Style::default().fg(Color::LightRed), " - Empty".to_string());
        }

        let file_name = format!("{}.desktop", trimmed);

        let save_path = dirs::data_local_dir()
            .unwrap()
            .join("applications")
            .join(&file_name);

        if save_path.exists() {
            return (
                Style::default().fg(Color::LightRed),
                " - Already exists".to_string(),
            );
        }

        (Style::default().fg(Color::LightGreen), " - OK".to_string())
    }
    pub fn is_active_block_style(&self, index: usize) -> Style {
        if self.block_index == index && index == IDX_CANCEL {
            Style::default().fg(Color::LightRed)
        } else if self.block_index == index {
            Style::default().fg(Color::LightGreen)
        } else {
            Style::default()
        }
    }

    pub fn checkbox(&mut self) {
        match self.block_index {
            IDX_NODISPLAY => self.checkbox_nodisplay = !self.checkbox_nodisplay,
            IDX_TERMINAL => self.checkbox_terminal = !self.checkbox_terminal,
            IDX_STARTUPNOTIFY => self.checkbox_startupnotify = !self.checkbox_startupnotify,
            _ => {}
        }
    }

    pub fn exit(&mut self) {
        self.exit = true;
    }
}
