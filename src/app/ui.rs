use crate::app::{state::App, state::InputMode};
use crate::utils::constants::*;

use color_eyre::eyre::{Ok, Result};

use ratatui::widgets::Wrap;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Line,
    widgets::{Block, List, ListItem, Paragraph},
};

impl App {
    pub fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_event()?;
        }

        Ok(())
    }

    pub fn draw(&self, frame: &mut Frame) {
        let area;
        let frame_width = frame.area().width;
        let frame_height = frame.area().height;

        if frame_width < SMALLEST_WIDTH || frame_height < SMALLEST_HEIGHT {
            let warning_layout = Layout::vertical([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Length(3),
                Constraint::Percentage(20),
            ])
            .split(frame.area());

            let current_area = warning_layout[1];
            let needed_area = warning_layout[2];
            let current = Paragraph::new(format!(
                "Terminal size is too small:\n Width: {} Height: {}",
                frame_width, frame_height
            ))
            .centered();

            let needed = Paragraph::new(format!(
                "Needed terminal size:\n Width: {} Height: {}",
                SMALLEST_WIDTH, SMALLEST_HEIGHT
            ))
            .centered();

            frame.render_widget(current, current_area);
            frame.render_widget(needed, needed_area);

            return;
        }

        let horizontal_margin = if frame_width > HALF_SCREEN { 20 } else { 0 };

        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .horizontal_margin(horizontal_margin);

        let [_instructions_area, outline_area, _desktop_name] = vertical.areas(frame.area());

        let (msg, _style) = match self.input_mode {
            InputMode::Normal => (
                vec!["Mode: ".into(), "NORMAL".white().bold()],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            InputMode::Insert => (
                vec!["Mode: ".into(), "INSERT".white().bold()],
                Style::default(),
            ),
        };

        let vim_mode = match self.input_mode {
            InputMode::Normal => {
                Line::from(Line::from(vec![" Insert: ".into(), "<I> ".white().bold()]))
            }

            InputMode::Insert => Line::from(Line::from(vec![
                " Normal: ".into(),
                "<Esc> ".white().bold(),
            ])),
        }
        .centered();

        let instructions = Line::from(
            Line::from(vec![
                " Next ".into(),
                "<J> ".white().bold(),
                "─".into(),
                " Previous ".into(),
                "<K> ".white().bold(),
                "─".into(),
                " Quit ".into(),
                "<Q> ".white().bold(),
            ])
            .centered(),
        );

        let outline_block = Block::bordered()
            .title_bottom(msg)
            .title_bottom(vim_mode)
            .title_bottom(instructions)
            .title(Line::from(
                " DeskForge - Create Launcher ".bold().into_centered_line(),
            ));

        frame.render_widget(&outline_block, outline_area);

        let inner = outline_block.inner(outline_area);
        let [required_area, optional_area, button_area] = *Layout::vertical([
            Constraint::Length(11),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(inner) else {
            unreachable!()
        };

        // Require & Optional area
        let requireed_block = Block::bordered().title("Required");
        let optional_block = Block::bordered().title("Optional");

        // Require & Block inner
        let required_inner = requireed_block.inner(required_area);
        let [name_area, exec_area, icon_area] = *Layout::vertical([
            Constraint::Length(3), // Name
            Constraint::Min(3),    // Exec
            Constraint::Length(3), // Icon
        ])
        .split(required_inner) else {
            unreachable!()
        };

        let optional_inner = optional_block.inner(optional_area);
        let [
            version_area,
            comment_area,
            action_area,
            nodisplay_area,
            startupnotify_area,
            terminal_area,
            type_area,
            category_area,
        ] = *Layout::vertical([
            Constraint::Length(2), // Command
            Constraint::Length(2), // Comment
            Constraint::Length(2), // Action
            Constraint::Length(2), // NoDisplay
            Constraint::Length(2), // StartUpNotify
            Constraint::Length(2), // Terminal
            Constraint::Length(2), // Type
            Constraint::Length(2), // Category
        ])
        .split(optional_inner)
        else {
            unreachable!()
        };

        frame.render_widget(requireed_block, required_area);
        frame.render_widget(optional_block, optional_area);

        // Button layout
        let buttons_area = Layout::horizontal([
            Constraint::Percentage(40),
            Constraint::Length(20),
            Constraint::Length(20),
            Constraint::Percentage(40),
        ])
        .split(button_area);

        // Desktop name block
        let (name_style, name_status) = self.validate_name(self.input[IDX_NAME].value(), IDX_NAME);
        let name = Paragraph::new(self.input[IDX_NAME].value())
            .style(name_style)
            .block(Block::bordered().title(format!("Name{}", name_status)))
            .add_modifier(Modifier::BOLD);
        frame.render_widget(name, name_area);

        // Exec & URL block
        let type_value = self.input[IDX_TYPE].value();
        let mut exec_or_url_area = Rect {
            x: exec_area.x,
            y: 6,
            width: exec_area.width.max(1),
            height: 3,
        };

        match type_value {
            "Link" => {
                let (url_style, url_status) =
                    self.validate_path(self.input[IDX_URL].value(), &[], IDX_URL);
                let url = Paragraph::new(self.input[IDX_URL].value())
                    .style(url_style)
                    .block(Block::bordered().title(format!("URL{}", url_status)))
                    .add_modifier(Modifier::BOLD);
                frame.render_widget(url, exec_or_url_area);
            }

            _ => {
                let input_len = self.input[IDX_EXEC].value().len();

                match self.block_index {
                    IDX_EXEC => {
                        exec_or_url_area.height =
                            ((input_len / (exec_or_url_area.width - 1) as usize) + 3) as u16;
                        if input_len >= (exec_or_url_area.width - 2) as usize {
                            if exec_or_url_area.height >= 6 {
                                exec_or_url_area.height = 6;
                            }
                        }
                    }
                    _ => {}
                }

                let (exec_color, exec_status) =
                    self.validate_path(self.input[IDX_EXEC].value(), &[], IDX_EXEC);
                let exec = Paragraph::new(self.input[IDX_EXEC].value())
                    .style(exec_color)
                    .block(
                        Block::bordered()
                            .title(format!("Exec{}", exec_status))
                            .border_style(exec_color),
                    )
                    .wrap(Wrap { trim: true })
                    .add_modifier(Modifier::BOLD);

                frame.render_widget(exec, exec_or_url_area);
            }
        }

        // Icon block
        if ((self.input[IDX_EXEC].value().len() / (exec_or_url_area.width - 1) as usize) + 3) < 4
            || self.block_index != IDX_EXEC
        {
            let (icon_style, icon_status) = self.validate_path(
                self.input[IDX_ICON].value(),
                &["png", "svg", "jpg"],
                IDX_ICON,
            );
            let icon = Paragraph::new(self.input[IDX_ICON].value())
                .style(icon_style)
                .block(
                    Block::bordered()
                        .title(format!("Icon{}", icon_status))
                        .border_style(icon_style),
                )
                .add_modifier(Modifier::BOLD);
            frame.render_widget(icon, icon_area);
        }

        // Command block
        let version_style = self.is_active_block_style(IDX_VERSION);
        let version = Paragraph::new(format!("Version: [ {}  ]", self.input[IDX_VERSION].value()))
            .style(version_style)
            .add_modifier(Modifier::BOLD);
        frame.render_widget(version, version_area);

        // Comment block
        let comment_style = self.is_active_block_style(IDX_COMMENT);
        let comment = Paragraph::new(format!("Comment: [ {}  ]", self.input[IDX_COMMENT].value()))
            .style(comment_style)
            .add_modifier(Modifier::BOLD);
        frame.render_widget(comment, comment_area);

        // Comment block
        let action_style = self.is_active_block_style(IDX_ACTION);
        let action = Paragraph::new(format!("Actions: [ {}  ]", self.input[IDX_ACTION].value()))
            .style(action_style)
            .add_modifier(Modifier::BOLD);
        frame.render_widget(action, action_area);

        // NoDisplay block
        let nodisplay_style = self.is_active_block_style(IDX_NODISPLAY);
        let nodisplay_label = if self.checkbox_nodisplay {
            "NoDisplay: [ X ]"
        } else {
            "NoDisplay: [   ]"
        };
        let nodisplay = Paragraph::new(nodisplay_label)
            .style(nodisplay_style)
            .add_modifier(Modifier::BOLD);
        frame.render_widget(nodisplay, nodisplay_area);

        // NoDisplay block
        let startupnotify_style = self.is_active_block_style(IDX_STARTUPNOTIFY);
        let startupnotify_label = if self.checkbox_startupnotify {
            "StartUpNotify: [ X ]"
        } else {
            "StartUpNotify: [   ]"
        };
        let startupnotify = Paragraph::new(startupnotify_label)
            .style(startupnotify_style)
            .add_modifier(Modifier::BOLD);
        frame.render_widget(startupnotify, startupnotify_area);

        // Terminal block
        let terminal_style = self.is_active_block_style(IDX_TERMINAL);
        let terminal_label = if self.checkbox_terminal {
            "Terminal: [ X ]"
        } else {
            "Terminal: [   ]"
        };
        let terminal = Paragraph::new(terminal_label)
            .style(terminal_style)
            .add_modifier(Modifier::BOLD);
        frame.render_widget(terminal, terminal_area);

        // Type
        let arrow = if self.dropdown_open && self.dropdown_index == Some(IDX_TYPE) {
            "▲"
        } else {
            "▼"
        };
        let type_label = format!("Type: [ {} {} ]", self.input[IDX_TYPE].value(), arrow);
        frame.render_widget(
            Paragraph::new(type_label).style(self.is_active_block_style(IDX_TYPE)),
            type_area,
        );

        // Category
        if !(self.dropdown_open && self.dropdown_index != Some(IDX_CATEGORY)) {
            let arrow = if self.dropdown_open && self.dropdown_index == Some(IDX_CATEGORY) {
                "▲"
            } else {
                "▼"
            };
            let category_label = format!(
                "Category: [ {} {} ]",
                self.input[IDX_CATEGORY].value(),
                arrow
            );
            frame.render_widget(
                Paragraph::new(category_label).style(self.is_active_block_style(IDX_CATEGORY)),
                category_area,
            );
        }

        if self.dropdown_open {
            let idx = self.dropdown_index.unwrap();
            let area = match idx {
                IDX_TYPE => type_area,
                IDX_CATEGORY => category_area,
                _ => return,
            };
            let dropdown_area = Rect {
                x: if self.dropdown_index == Some(IDX_TYPE) {
                    area.x + 8
                } else {
                    area.x + 12
                },
                y: area.y + area.height,
                width: area.width,
                height: self.dropdown_options.len() as u16,
            };
            let items: Vec<ListItem> = self
                .dropdown_options
                .iter()
                .enumerate()
                .map(|(i, option)| {
                    let style = if i == self.dropdown_selected {
                        Style::default().fg(Color::LightGreen)
                    } else {
                        Style::default()
                    };
                    ListItem::new(*option).style(style)
                })
                .collect();

            frame.render_widget(List::new(items).block(Block::default()), dropdown_area);
        }

        // Buttons
        // let save_style = if self.can_save() {
        //     self.is_active_block_style(IDX_SAVE)
        // } else if !self.can_save() && self.block_index == IDX_SAVE {
        //     Style::default().fg(Color::LightRed)
        // } else {
        //     Style::default()
        // };
        // let save_label = if self.can_save() {
        //     "[ SAVE ]"
        // } else {
        //     "[ CAN'T SAVE ]"
        // };
        let save_style = self.is_active_block_style(IDX_SAVE);
        let save_label = "[ SAVE ]";
        let save_btn = Paragraph::new(format!("{}", save_label))
            .style(save_style)
            .add_modifier(Modifier::BOLD)
            .alignment(ratatui::layout::Alignment::Center);

        let cancel_style = self.is_active_block_style(IDX_CANCEL);
        let cancel_btn = Paragraph::new("[ CANCEL ]")
            .style(cancel_style)
            .add_modifier(Modifier::BOLD)
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(save_btn, buttons_area[1]);
        frame.render_widget(cancel_btn, buttons_area[2]);

        // Insert mode
        match self.block_index {
            IDX_NAME => area = name_area,
            IDX_EXEC => area = exec_area,
            IDX_ICON => area = icon_area,

            IDX_VERSION => area = version_area,
            IDX_COMMENT => area = comment_area,
            IDX_ACTION => area = action_area,
            IDX_NODISPLAY => return,
            IDX_STARTUPNOTIFY => return,
            IDX_TERMINAL => return,
            IDX_TYPE => return,
            IDX_CATEGORY => return,

            IDX_SAVE => return,
            IDX_CANCEL => return,
            _ => area = name_area,
        }

        match self.input_mode {
            InputMode::Normal => {}
            InputMode::Insert => {
                let (area_x, area_y): (u16, u16) = if self.block_index >= IDX_VERSION {
                    (10, 0)
                } else {
                    (0, 1)
                };
                
                let input_len = self.input[IDX_EXEC].value().len();
                let current_cursor = self.input[self.block_index].visual_cursor() as u16;
                let mut cursor_x = area.x + area_x + current_cursor + 1;
                let mut cursor_y = area.y.saturating_add(area_y);

                if self.block_index == IDX_EXEC {
                    let width = exec_or_url_area.width.saturating_sub(1);
                    let scale = ((input_len / (exec_or_url_area.width - 1) as usize)) as u16;
                    let line_index = current_cursor / width;
                    let column_index = current_cursor % width;

                    cursor_x = area.x + area_x + column_index + scale + 1;
                    cursor_y = area.y + area_y + line_index;

                    cursor_x = cursor_x.min(area.x + exec_or_url_area.width - 1);
                    cursor_y = cursor_y.min(area.y + exec_or_url_area.height - 1);
                }

                #[allow(clippy::cast_possible_truncation)]
                frame.set_cursor_position(Position::new(cursor_x, cursor_y))
            }
        }
    }
}
