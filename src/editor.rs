use std::io::{self};

use crossterm::{
    cursor::MoveTo,
    event::{read, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    style::Print,
    terminal::{self, ClearType},
};

#[derive(Clone, Copy)]
pub struct CursorPosition {
    x: u16,
    y: u16,
}

impl CursorPosition {
    fn new() -> Self {
        Self { x: 0, y: 0 }
    }
}

pub struct Editor {
    buffer: Vec<String>,
    cursor_position: CursorPosition,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            buffer: vec![String::new()],
            cursor_position: CursorPosition::new(),
        }
    }

    pub fn render(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(stdout, terminal::Clear(ClearType::All))?;

        for (y, line) in self.buffer.iter().enumerate() {
            execute!(stdout, MoveTo(0, y as u16), Print(line))?;
        }

        execute!(
            stdout,
            MoveTo(self.cursor_position.x, self.cursor_position.y)
        )
    }
    fn insert_char(&mut self, c: char) {
        let current_line_index = self.cursor_position.y as usize;
        let initial_cursor_horizontal_position = self.cursor_position.x as usize;
        let current_line = &mut self.buffer[current_line_index];

        current_line.insert(initial_cursor_horizontal_position, c);
        self.cursor_position.x += 1;
    }
    fn new_line(&mut self) {
        let current_line_index = self.cursor_position.y as usize;
        let initial_cursor_horizontal_position = self.cursor_position.x as usize;

        let new_line_content =
            self.buffer[current_line_index].split_off(initial_cursor_horizontal_position);

        self.buffer.insert(current_line_index + 1, new_line_content);

        self.cursor_position.x = 0;
        self.cursor_position.y += 1;
    }
    fn backspace(&mut self) {
        let current_line_index = self.cursor_position.y as usize;
        let initial_cursor_horizontal_position = self.cursor_position.x as usize;

        if initial_cursor_horizontal_position > 0 {
            let current_line = &mut self.buffer[current_line_index];

            current_line.remove(self.cursor_position.x as usize - 1);
            self.cursor_position.x -= 1
        } else if current_line_index > 0 {
            let prev_line_index = current_line_index - 1;
            let prev_line_len = self.buffer[prev_line_index].len() as u16;
            let current_line = self.buffer.remove(current_line_index);

            self.buffer[prev_line_index].push_str(&current_line);
            self.cursor_position.y -= 1;
            self.cursor_position.x = prev_line_len;
        }
    }
}
