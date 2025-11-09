use color_eyre::{
    eyre::{Ok, Result},
    owo_colors::OwoColorize,
};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Position},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, Paragraph},
};

#[derive(Debug)]
pub struct App {
    input: String,
    character_index: usize,
    input_mode: InputMode,
    message: Vec<String>,
    exit: bool,
}

#[derive(Debug)]
enum InputMode {
    Normal,
    Insert,
}

impl App {
    const fn new() -> Self {
        Self {
            input: String::new(),
            character_index: 0,
            input_mode: InputMode::Normal,
            message: Vec::new(),
            exit: false,
        }
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input.chars().skip(current_index);

            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn submit_message(&mut self) {
        self.message.push(self.input.clone());
        self.input.clear();
        self.reset_cursor();
    }

    fn hand_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_input(key_event)
            }
            _ => {}
        };

        Ok(())
    }

    fn handle_key_input(&mut self, key_event: KeyEvent) {
        match self.input_mode {
            InputMode::Normal => match key_event.code {
                KeyCode::Esc | KeyCode::Char('q') => self.exit(),
                KeyCode::Char('i') => self.input_mode = InputMode::Insert,
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
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(3),
        ]);

        let [instructions_area, title_area, desktop_name] = vertical.areas(frame.area());

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

            InputMode::Insert => {
                Line::from(Line::from(vec![" Normal: ".into(), "<Esc> ".white().bold()]))
            }
        }
        .centered();

        let instructions =
            Line::from(Line::from(vec![" Quit ".into(), "<Q> ".white().bold()]).centered());

        frame.render_widget(
            Block::bordered()
                .title_bottom(msg)
                .title_bottom(vim_mode)
                .title_bottom(instructions)
                .title(Line::from(
                    " DeskForge - Create Launcher ".bold().into_centered_line(),
                )),
            title_area,
        );
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut terminal = ratatui::init();
    let result = App::new().run(&mut terminal);

    ratatui::restore();
    result
}
