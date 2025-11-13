mod cli;

use clap::Parser;
use cli::Cli;

use color_eyre::{
    eyre::{Ok, Result},
    owo_colors::OwoColorize,
};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, Paragraph, block},
};
use tui_input::{Input, backend::crossterm::EventHandler};

use is_executable::IsExecutable;

use std::{
    env::consts::EXE_EXTENSION,
    fmt::format,
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
    str::RSplitTerminator,
    sync::WaitTimeoutResult,
};

const HALF_SCREEN: u16 = 89;
const SMALLEST_WIDTH: u16 = 41;
const SMALLEST_HEIGHT: u16 = 18;

const NUM_BLOCK: usize = 11;

const IDX_NAME: usize = 0;

const IDX_EXEC: usize = 1;
const IDX_URL: usize = 1;

const IDX_ICON: usize = 2;
const IDX_COMMAND: usize = 3;
const IDX_COMMENT: usize = 4;
const IDX_NODISPLAY: usize = 5;
const IDX_TERMINAL: usize = 6;
const IDX_TYPE: usize = 7;
const IDX_CATEGORY: usize = 8;
const IDX_SAVE: usize = 9;
const IDX_CANCEL: usize = 10;

/*
 * 0: Name
 * 1: Exec
 * 2: Icon
 * 3: Command
 * 4: Comment
 * 5: Terminal
 * 6: Type
 * 7: Category
 * 8: Save
 * 9: Cancel
 */

#[derive(Debug)]
pub struct App {
    input_mode: InputMode,
    input: Vec<Input>,
    last_key: Option<KeyCode>,

    dropdown_open: bool,
    dropdown_options: Vec<&'static str>,
    dropdown_selected: usize,
    dropdown_index: Option<usize>,

    block_index: usize,

    checkbox_nodisplay: bool,
    checkbox_terminal: bool,
    exit: bool,
}

#[derive(Debug)]
enum InputMode {
    Normal,
    Insert,
}

impl App {
    fn new(file_name: Option<String>) -> Self {
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

    fn next_block(&mut self) {
        if self.block_index == IDX_CANCEL {
            return;
        }
        self.block_index += 1;
    }

    fn previous_block(&mut self) {
        // Name
        if self.block_index == IDX_NAME {
            return;
        }
        self.block_index -= 1;
    }

    fn submit_message(&mut self) {
        if self.block_index == IDX_TERMINAL || self.block_index == IDX_NODISPLAY {
            self.checkbox();
        }
        self.next_block();
    }

    fn open_dropdown(&mut self, index: usize, options: Vec<&'static str>) {
        self.dropdown_open = true;
        self.dropdown_options = options;
        self.dropdown_selected = 0;
        self.dropdown_index = Some(index);
        self.input[index] = Input::from(self.dropdown_options[0]);
    }

    fn validate_path(&self, input: &str, exts: &[&str], index: usize) -> (Style, String) {
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

        if !path.exists() {
            return (
                Style::default().fg(Color::LightRed),
                "- Not found".to_string(),
            );
        }

        if self.input[IDX_TYPE].value().eq("Link") {
            if trimmed.is_empty() {
                return (Style::default().fg(Color::LightRed), " - Empty".to_string());
            }

            if !(trimmed.starts_with("files://")
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
                            "- File not found".to_string(),
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

    fn can_save(&self) -> bool {
        let exec_path = Path::new(self.input[IDX_EXEC].value());
        let exec_ok = exec_path.exists();

        exec_ok
    }

    fn handle_event(&mut self) -> Result<()> {
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
                    IDX_TERMINAL | IDX_NODISPLAY => self.checkbox(),
                    IDX_TYPE => {
                        self.open_dropdown(IDX_TYPE, vec!["Application", "Link", "Directory"]);
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
                        let file_name = format!("{}.desktop", self.input[IDX_NAME].value());
                        self.save_as_desktop(&file_name).expect("CANNOT SAVE!");
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

    fn is_active_block_style(&self, index: usize) -> Style {
        if self.block_index == index && index == IDX_CANCEL {
            Style::default().fg(Color::LightRed)
        } else if self.block_index == index {
            Style::default().fg(Color::LightGreen)
        } else {
            Style::default()
        }
    }

    fn checkbox(&mut self) {
        match self.block_index {
            IDX_NODISPLAY => self.checkbox_nodisplay = !self.checkbox_nodisplay,
            IDX_TERMINAL => self.checkbox_terminal = !self.checkbox_terminal,
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn save_as_desktop(&self, file_name: &str) -> Result<()> {
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

    fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_event()?;
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
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
            Constraint::Length(3), // Exec
            Constraint::Length(3), // Icon
        ])
        .split(required_inner) else {
            unreachable!()
        };

        let optional_inner = optional_block.inner(optional_area);
        let [
            command_area,
            comment_area,
            nodisplay_area,
            terminal_area,
            type_area,
            category_area,
        ] = *Layout::vertical([
            Constraint::Length(2), // Command
            Constraint::Length(2), // Comment
            Constraint::Length(2), // NoDisplay
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
        let name_style = self.is_active_block_style(IDX_NAME);
        let name = Paragraph::new(self.input[IDX_NAME].value())
            .style(name_style)
            .block(Block::bordered().title("Name"))
            .add_modifier(Modifier::BOLD);
        frame.render_widget(name, name_area);

        let type_value = self.input[IDX_TYPE].value();
        let exec_or_url_area = exec_area;

        match type_value {
            "Link" => {
                let url_style = self.is_active_block_style(IDX_URL);
                let url = Paragraph::new(self.input[IDX_URL].value())
                    .style(url_style)
                    .block(Block::bordered().title("URL"))
                    .add_modifier(Modifier::BOLD);
                frame.render_widget(url, exec_or_url_area);
            }

            // Exec block
            _ => {
                let (exec_color, exec_status) =
                    self.validate_path(self.input[IDX_EXEC].value(), &[], IDX_EXEC);
                let exec = Paragraph::new(self.input[IDX_EXEC].value())
                    .style(exec_color)
                    .block(
                        Block::bordered()
                            .title(format!("Exec{}", exec_status))
                            .border_style(exec_color),
                    )
                    .add_modifier(Modifier::BOLD);
                frame.render_widget(exec, exec_or_url_area);
            }
        }
        // URL block

        // Icon block
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

        // Command block
        let command_style = self.is_active_block_style(IDX_COMMAND);
        let command = Paragraph::new(format!("Command: [ {}  ]", self.input[IDX_COMMAND].value()))
            .style(command_style)
            .add_modifier(Modifier::BOLD);
        frame.render_widget(command, command_area);

        // Comment block
        let comment_style = self.is_active_block_style(IDX_COMMENT);
        let comment = Paragraph::new(format!("Comment: [ {}  ]", self.input[IDX_COMMENT].value()))
            .style(comment_style)
            .add_modifier(Modifier::BOLD);
        frame.render_widget(comment, comment_area);

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
        let save_style = if self.can_save() {
            self.is_active_block_style(IDX_SAVE)
        } else if !self.can_save() && self.block_index == IDX_SAVE {
            Style::default().fg(Color::LightRed)
        } else {
            Style::default()
        };
        let save_label = if self.can_save() {
            "[ SAVE ]"
        } else {
            "[ CAN'T SAVE ]"
        };
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

            IDX_COMMAND => area = command_area,
            IDX_COMMENT => area = comment_area,
            IDX_NODISPLAY => return,
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
                let (area_x, area_y): (u16, u16) = if self.block_index >= IDX_COMMAND {
                    (10, 0)
                } else {
                    (0, 1)
                };

                let cursor_x = area.x + area_x;
                let cursor_y = area.y.saturating_add(area_y);

                #[allow(clippy::cast_possible_truncation)]
                frame.set_cursor_position(Position::new(
                    cursor_x + self.input[self.block_index].visual_cursor() as u16 + 1,
                    cursor_y,
                ))
            }
        }
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    if let Some(name) = cli.new {
        let file_name = name.unwrap_or_default();
        let mut terminal = ratatui::init();
        let result = App::new(Some(file_name)).run(&mut terminal);
        ratatui::restore();
        return result;
    }

    if let Some(file_name) = cli.edit {
        let mut terminal = ratatui::init();
        let result = App::new(Some(file_name)).run(&mut terminal);
        ratatui::restore();
        return result;
    } else {
        Ok(())
    }
}
