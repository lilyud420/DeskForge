use crate::utils::constants::*;

use color_eyre::eyre::Ok;
use color_eyre::eyre::Result;
use is_executable::IsExecutable;
use ratatui::{
    crossterm::event::KeyCode,
    style::{Color, Style},
};
use tui_input::Input;

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
    pub checkbox_terminal: bool,
    pub exit: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Insert,
}

impl App {
    pub fn new(file_name: Option<String>) -> Self {
        let mut input = vec![Input::default(); NUM_BLOCK];
        let mut block_index: usize = 0;

        input[IDX_TYPE] = Input::from("Application");
        input[IDX_CATEGORY] = Input::from("None");

        if let Some(name) = file_name
            && name != ""
        {
            input[0] = Input::from(name);
            block_index += 1;
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
            checkbox_terminal: false,
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
        if self.block_index == IDX_TERMINAL || self.block_index == IDX_NODISPLAY {
            self.checkbox();
        }
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

        let exec_line = if self.input[IDX_COMMAND].value().trim().starts_with("env ") {
            let parts: Vec<&str> = self.input[IDX_COMMAND].value().split_whitespace().collect();
            if let Some((_env_part, rest)) = parts.split_first() {
                let mut iter = rest.iter();
                let mut env_vars = vec![];
                let mut flags = vec![];
                while let Some(s) = iter.next() {
                    if s.contains('=') {
                        env_vars.push(*s);
                    } else {
                        flags.push(*s);
                        flags.extend(iter);
                        break;
                    }
                }
                format!(
                    "env {} {} {}",
                    env_vars.join(" "),
                    self.input[IDX_EXEC].value(),
                    flags.join(" ")
                )
            } else {
                format!("{} {}", self.input[IDX_COMMAND], self.input[IDX_EXEC])
            }
        } else {
            format!("{} {}", self.input[IDX_COMMAND], self.input[IDX_EXEC])
        };

        writeln!(file, "Exec={}", exec_line)?;

        match self.input[IDX_TYPE].value() {
            "Link" => writeln!(file, "URL={}", self.input[IDX_URL])?,
            "Application" => writeln!(
                file,
                "Exec={} {}",
                self.input[IDX_COMMAND], self.input[IDX_EXEC]
            )?,

            "Directory" => todo!(),
            _ => {}
        }

        writeln!(file, "Icon={}", self.input[IDX_ICON])?;
        writeln!(file, "Comment={}", self.input[IDX_COMMENT])?;
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
                    || url.starts_with("mailto:")
                    || url.starts_with("smb://")
                    || url.starts_with("trash:///")
                    || url.starts_with("recent:///")
            }
            _ => {
                let exec_path = Path::new(self.input[IDX_EXEC].value());
                let exec_ok = exec_path.exists();

                exec_ok
            }
        }
    }

    pub fn validate_path(&self, input: &str, exts: &[&str], index: usize) -> (Style, String) {
        let trimmed = input.trim();
        let path = Path::new(trimmed);

        if self.block_index != index {
            return (Style::default(), "".to_string());
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

        if exts.is_empty() {
            if !path.is_executable() {
                return (
                    Style::default().fg(Color::Yellow),
                    " - Unexpected type".to_string(),
                );
            }
        }

        if !path.exists() {
            return (
                Style::default().fg(Color::LightRed),
                "- Not found".to_string(),
            );
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
            _ => {}
        }
    }

    pub fn exit(&mut self) {
        self.exit = true;
    }
}
