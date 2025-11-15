use crate::app::state::{App, InputMode};
use crate::utils::constants::*;

use color_eyre::eyre::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use color_eyre::eyre::Ok;

use tui_input::{Input, backend::crossterm::EventHandler};

impl App {
    pub fn handle_event(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_input(key_event)
            }
            Event::Resize(_, _) => {}
            _ => {}
        };
        Ok(())
    }

    fn handle_key_input(&mut self, key_event: KeyEvent) {
        match self.input_mode {
            InputMode::Normal => match key_event.code {
                KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.exit();
                }

                // Vim keys
                KeyCode::Char('g') => {
                    if let Some(KeyCode::Char('g')) = self.last_key {
                        self.block_index = IDX_NAME;
                        self.last_key = None;
                    } else {
                        self.last_key = Some(KeyCode::Char('g'));
                    }
                }
                KeyCode::Char('G') => {
                    if let Some(KeyCode::Char('g')) = self.last_key {
                        self.block_index = IDX_SAVE;
                    }
                    self.block_index = IDX_SAVE;
                    self.last_key = None;
                }
                KeyCode::Char('d') => {
                    if let Some(KeyCode::Char('d')) = self.last_key {
                        self.input[self.block_index].value_and_reset();
                        self.last_key = None;
                    } else {
                        self.last_key = Some(KeyCode::Char('d'));
                    }
                }

                KeyCode::Char('j') | KeyCode::Down => {
                    self.next_block();
                }

                KeyCode::Char('k') | KeyCode::Up => {
                    self.previous_block();
                }

                KeyCode::Char('q') => {
                    self.exit();
                }

                KeyCode::Char('i') => match self.block_index {
                    IDX_TERMINAL | IDX_NODISPLAY | IDX_STARTUPNOTIFY => {
                        self.checkbox();
                        self.next_block();
                    }
                    IDX_TYPE => {
                        self.open_dropdown(
                            IDX_TYPE,
                            vec!["Application", "Application (other)", "Link", "Directory"],
                        );
                        self.input_mode = InputMode::Insert;
                    }

                    IDX_CATEGORY => {
                        self.open_dropdown(
                            IDX_CATEGORY,
                            vec![
                                "None",
                                "Audio",
                                "Video",
                                "Development",
                                "Education",
                                "Graphics",
                                "Network",
                                "Office",
                                "Settings",
                                "System",
                            ],
                        );
                        self.input_mode = InputMode::Insert;
                    }
                    IDX_SAVE => {
                        if !self.can_save() {
                            return;
                        }
                        let file_name = format!("{}.desktop", self.input[IDX_NAME].value());
                        self.save_as_desktop(&file_name)
                            .expect("[ERROR]: Can't save!");
                        self.exit();
                    }
                    IDX_CANCEL => self.exit(),
                    _ => self.input_mode = InputMode::Insert,
                },

                _ => self.last_key = None,
            },

            InputMode::Insert => {
                if self.block_index != IDX_TYPE && self.block_index != IDX_CATEGORY {
                    self.input[self.block_index].handle_event(&Event::Key(key_event));
                }

                // tHIS IS A PIECE OF SHIT, HELP M,E
                if let Some(idx) = self.dropdown_index {
                    if key_event.code == KeyCode::Down || key_event.code == KeyCode::Char('j') {
                        self.dropdown_selected =
                            (self.dropdown_selected + 1) % self.dropdown_options.len();
                        self.input[idx] =
                            Input::from(self.dropdown_options[self.dropdown_selected]);
                    }
                    if key_event.code == KeyCode::Up || key_event.code == KeyCode::Char('k') {
                        self.dropdown_selected = self.dropdown_selected.saturating_sub(1);
                        self.input[idx] =
                            Input::from(self.dropdown_options[self.dropdown_selected]);
                    }
                    if key_event.code == KeyCode::Enter || key_event.code == KeyCode::Char('i') {
                        self.dropdown_open = false;
                        self.dropdown_index = None;
                        self.submit_message();
                        self.input_mode = InputMode::Normal;
                    }
                }

                if key_event.code == KeyCode::Esc {
                    match self.block_index {
                        IDX_TYPE | IDX_CATEGORY => {
                            self.dropdown_open = false;
                            self.dropdown_index = None;
                            self.input_mode = InputMode::Normal;
                        }
                        _ => self.input_mode = InputMode::Normal,
                    }
                }

                if key_event.code == KeyCode::Enter {
                    match self.block_index {
                        IDX_COMMENT => {
                            self.submit_message();
                            self.input_mode = InputMode::Normal;
                        }
                        IDX_TYPE => return,
                        IDX_CATEGORY => return,
                        _ => self.submit_message(),
                    }
                }
            }
        }
    }
}
