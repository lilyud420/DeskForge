mod cli;

use clap::Parser;
use cli::Cli;

mod utils;
use crate::utils::shrink_rect::shrink;

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
use tui_checkbox::Checkbox;

const NUM_BLOCK: usize = 7;

#[derive(Debug)]
pub struct App {
    character_index: Vec<usize>,
    block_index: usize,
    input_mode: InputMode,
    input: Vec<String>,
    message: Vec<String>,
    checkbox: bool,
    exit: bool,
}

#[derive(Debug)]
enum InputMode {
    Normal,
    Insert,
}

impl App {
    fn new(file_name: Option<String>) -> Self {
        let mut input = vec![String::new(); NUM_BLOCK];
        let mut character_index = vec![0; NUM_BLOCK];
        let mut block_index: usize = 0;

        if let Some(name) = file_name
            && name != ""
        {
            input[0] = name;
            character_index[0] = input[0].chars().count();
            block_index += 1;
        }

        Self {
            character_index,
            block_index,
            checkbox: false,
            input_mode: InputMode::Normal,
            input,
            message: Vec::new(),
            exit: false,
        }
    }

    fn current_input(&self) -> &String {
        &self.input[self.block_index]
    }

    fn current_input_mut(&mut self) -> &mut String {
        &mut self.input[self.block_index]
    }

    fn current_cursor(&self) -> usize {
        self.character_index[self.block_index]
    }

    fn current_cursor_mut(&mut self) -> &mut usize {
        &mut self.character_index[self.block_index]
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.current_cursor().saturating_sub(1);
        *self.current_cursor_mut() = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.current_cursor().saturating_add(1);
        *self.current_cursor_mut() = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.current_input_mut().insert(index, new_char);
        self.move_cursor_right();
    }

    fn byte_index(&self) -> usize {
        self.current_input()
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.current_cursor())
            .unwrap_or(self.current_input().len())
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.current_input().chars().count())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.current_cursor() != 0;
        if is_not_cursor_leftmost {
            let current_index = self.current_cursor();
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self
                .current_input()
                .chars()
                .take(from_left_to_current_index);
            let after_char_to_delete = self.current_input().chars().skip(current_index);

            let new_string: String = before_char_to_delete.chain(after_char_to_delete).collect();
            *self.current_input_mut() = new_string;
            self.move_cursor_left();
        }
    }

    fn reset_cursor(&mut self) {
        *self.current_cursor_mut() = 0;
    }

    fn next_block(&mut self) {
        if self.block_index == 6 {
            return;
        }
        self.block_index += 1;
    }

    fn previous_block(&mut self) {
        if self.block_index == 0 {
            return;
        }
        self.block_index -= 1;
    }

    fn submit_message(&mut self) {
        self.message.push(self.current_input().clone());
        // *self.current_input_mut() = String::new();
        self.next_block();
        self.reset_cursor();
    }

    fn hand_events(&mut self) -> Result<()> {
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
                KeyCode::Char('q') => self.exit(),
                KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                   self.exit(); 
                },
                KeyCode::Char('i') => match self.block_index {
                    4 => self.checkbox(),
                    _ => self.input_mode = InputMode::Insert,
                },
                KeyCode::Char('j') | KeyCode::Down => self.next_block(),
                KeyCode::Char('k') | KeyCode::Up => self.previous_block(),
                KeyCode::Enter => {
                    if self.block_index == 4 {
                        self.checkbox();
                    }
                }
                _ => {}
            },
            InputMode::Insert if key_event.kind == KeyEventKind::Press => match key_event.code {
                KeyCode::Enter => self.submit_message(),
                KeyCode::Char(to_insert) => self.enter_char(to_insert),
                KeyCode::Backspace => self.delete_char(),
                KeyCode::Left => self.move_cursor_left(),
                KeyCode::Right => self.move_cursor_right(),
                KeyCode::Esc => self.input_mode = InputMode::Normal,
                _ => {}
            },
            InputMode::Insert => {}
        }
    }

    fn _is_active_block(&self, index: usize) -> bool {
        matches!(self.input_mode, InputMode::Insert) && self.block_index == index
    }

    fn is_active_block_style(&self, index: usize) -> Style {
        if self.block_index == index {
            Style::default().fg(Color::LightGreen)
        } else {
            Style::default()
        }
    }

    fn checkbox(&mut self) {
        self.checkbox = !self.checkbox;
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.hand_events()?;
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let area;

        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .horizontal_margin(20);

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
        let [required_area, optional_area] =
            *Layout::vertical([Constraint::Length(11), Constraint::Min(0)]).split(inner)
        else {
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
        let [comment_area, terminal_area, type_area, category_area] = *Layout::vertical([
            Constraint::Length(2), // Comment
            Constraint::Length(2), // Terminal
            Constraint::Length(2), // Type
            Constraint::Length(2), // Category
        ])
        .split(optional_inner) else {
            unreachable!()
        };

        frame.render_widget(requireed_block, required_area);
        frame.render_widget(optional_block, optional_area);

        // Desktop name block
        let name_style = self.is_active_block_style(0);
        let name = Paragraph::new(self.input[0].as_str())
            .style(name_style)
            .block(Block::bordered().title("Name"));
        frame.render_widget(name, name_area);
        
        // Exec block
        let exec_style = self.is_active_block_style(1);
        let exec = Paragraph::new(self.input[1].as_str())
            .style(exec_style)
            .block(Block::bordered().title("Exec"));
        frame.render_widget(exec, exec_area);

        // Icon block
        let icon_style = self.is_active_block_style(2);
        let icon = Paragraph::new(self.input[2].as_str())
            .style(icon_style)
            .block(Block::bordered().title("Icon"));
        frame.render_widget(icon, icon_area);

        // Comment block
        // let comment_style = self.is_active_block_style(3);
        // let comment = Paragraph::new(self.input[3].as_str())
        //     .style(comment_style)
        //     .block(Block::bordered().title("Comment"));
        let comment_style = self.is_active_block_style(3);
        let comment =
            Paragraph::new(format!("Comment: [ {} ]", self.input[3].as_str())).style(comment_style);
        frame.render_widget(comment, comment_area);

        // Terminal block
        let terminal_style = self.is_active_block_style(4);
        // let terminal = Checkbox::new("Terminal", self.checkbox)
        //     .checked_symbol("[X]")
        //     .unchecked_symbol("[ ]")
        //     .style(terminal_style);
        let terminal_label = if self.checkbox {
            "Terminal: [ X ]"
        } else {
            "Terminal: [   ]"
        };
        let terminal = Paragraph::new(terminal_label).style(terminal_style);
        frame.render_widget(terminal, terminal_area);

        // Insert mode
        match self.block_index {
            0 => area = name_area,
            1 => area = exec_area,
            2 => area = icon_area,
            3 => area = comment_area,
            4 => return,
            5 => todo!(),
            6 => todo!(),
            _ => area = name_area,
        }

        match self.input_mode {
            InputMode::Normal => {}
            InputMode::Insert => {
                // This code is shit
                let (area_x, area_y): (u16, u16) = if self.block_index >= 3 {
                    (10, 0)
                } else {
                    (0, 1)
                };

                let cursor_x = area.x + area_x;
                let cursor_y = area.y.saturating_add(area_y);

                #[allow(clippy::cast_possible_truncation)]
                frame.set_cursor_position(Position::new(
                    cursor_x + self.character_index[self.block_index] as u16 + 1,
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
