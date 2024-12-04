use std::{
    fs::File,
    io::{self, Read, Write},
    path::Path,
};

use crossterm::{
    cursor::MoveTo,
    event::{read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    style::Print,
    terminal::{self, disable_raw_mode, enable_raw_mode, ClearType},
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
    filename: Option<String>,
    modified: bool,
    viewport_start: usize,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            buffer: vec![String::new()],
            cursor_position: CursorPosition::new(),
            filename: None,
            modified: false,
            viewport_start: 0,
        }
    }

    pub fn load_from_file(&mut self, filename: &str) -> io::Result<()> {
        let path = Path::new(filename);

        if path.exists() {
            let mut file = File::open(filename)?;
            let mut contents = String::new();

            file.read_to_string(&mut contents)?;

            self.buffer = contents.lines().map(String::from).collect();
            self.filename = Some(filename.to_string());
            self.modified = false;
        }

        Ok(())
    }

    pub fn render(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(stdout, terminal::Clear(ClearType::All))?;

        let terminal_height = terminal::size()?.1 as usize;
        let viewport_end = self.viewport_start + terminal_height;

        let viewport_end = std::cmp::min(viewport_end, self.buffer.len());

        for (y, line) in self.buffer[self.viewport_start..viewport_end]
            .iter()
            .enumerate()
        {
            execute!(stdout, MoveTo(0, y as u16), Print(line))?;
        }

        execute!(
            stdout,
            MoveTo(
                self.cursor_position.x,
                (self.cursor_position.y as usize - self.viewport_start) as u16
            )
        )?;

        self.render_scrollbar(stdout, terminal_height)?;

        Ok(())
    }

    fn render_scrollbar(&self, stdout: &mut io::Stdout, terminal_height: usize) -> io::Result<()> {
        let terminal_width = terminal::size()?.0 as usize;
        let buffer_length = self.buffer.len();

        if buffer_length <= terminal_height {
            return Ok(());
        }

        let scrollbar_height =
            terminal_height as f32 * (terminal_height as f32 / buffer_length as f32);
        let scrollbar_start =
            (self.viewport_start as f32 / buffer_length as f32) * terminal_height as f32;

        for y in 0..terminal_height {
            let y = y as f32;

            if y >= scrollbar_start && y < scrollbar_start + scrollbar_height {
                execute!(
                    stdout,
                    MoveTo((terminal_width - 1) as u16, y as u16),
                    Print("█")
                )?;
            } else {
                execute!(
                    stdout,
                    MoveTo((terminal_width - 1) as u16, y as u16),
                    Print("░")
                )?;
            }
        }

        Ok(())
    }

    pub fn handle_input(&mut self, stdout: &mut io::Stdout) -> io::Result<bool> {
        if let Event::Key(KeyEvent {
            code,
            kind,
            modifiers,
            ..
        }) = read()?
        {
            if kind == KeyEventKind::Press {
                match code {
                    KeyCode::Char(c)
                        if modifiers.contains(KeyModifiers::CONTROL)
                            && modifiers.contains(KeyModifiers::SHIFT) =>
                    {
                        match c {
                            's' => self.prompt_and_save_as(stdout)?,
                            _ => {}
                        }
                    }
                    KeyCode::Char(c) if modifiers.contains(KeyModifiers::CONTROL) => {
                        match c {
                            's' => {
                                if let Some(ref filename) = self.filename {
                                    self.save_to_file(filename)?;
                                } else {
                                    self.prompt_and_save_as(stdout)?;
                                }
                            }
                            _ => {}
                        }

                        self.modified = false;
                    }
                    KeyCode::Char(c) => {
                        self.insert_char(c);
                        self.modified = true
                    }
                    KeyCode::Enter => {
                        self.new_line();
                        self.modified = true;
                    }
                    KeyCode::Backspace => {
                        self.backspace();
                        self.modified = true;
                    }
                    KeyCode::Left => self.move_left(),
                    KeyCode::Right => self.move_right(),
                    KeyCode::Up => self.move_up(),
                    KeyCode::Down => self.move_down(),
                    KeyCode::Esc => {
                        if self.modified {
                            return self.prompt_if_save(stdout);
                        } else {
                            return Ok(false);
                        }
                    }
                    _ => {}
                }
            }
        }
        self.render(stdout)?;

        Ok(true)
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

        if let Ok((_, terminal_height)) = terminal::size() {
            let terminal_height = terminal_height as usize;

            if self.cursor_position.y >= ((self.viewport_start + terminal_height) as u16) {
                self.viewport_start = (self.cursor_position.y as usize) - terminal_height + 1
            }
        }
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

            if (self.cursor_position.y as usize) < self.viewport_start {
                self.viewport_start = self.cursor_position.y as usize;
            }
        }
    }

    fn move_left(&mut self) {
        if self.cursor_position.x > 0 {
            self.cursor_position.x -= 1;
        }
    }

    fn move_right(&mut self) {
        let current_line_index = self.cursor_position.y as usize;
        let initial_cursor_horizontal_position = self.cursor_position.x as usize;

        let current_line = &self.buffer[current_line_index];

        if initial_cursor_horizontal_position < current_line.len() {
            self.cursor_position.x += 1;
        }
    }

    fn move_up(&mut self) {
        let current_line_index = self.cursor_position.y as usize;
        let initial_cursor_horizontal_position = self.cursor_position.x as usize;

        if current_line_index > 0 {
            let prev_line_index = current_line_index - 1;
            let prev_line_length = self.buffer[prev_line_index].len();

            if initial_cursor_horizontal_position > prev_line_length {
                self.cursor_position.x = prev_line_length as u16;
            }

            self.cursor_position.y -= 1;

            if self.cursor_position.y < self.viewport_start as u16 {
                self.viewport_start = self.cursor_position.y as usize;
            }
        }
    }

    fn move_down(&mut self) {
        let current_line_index = self.cursor_position.y as usize;
        let initial_cursor_horizontal_position = self.cursor_position.x as usize;

        if current_line_index < self.buffer.len() - 1 {
            let next_line_index = current_line_index + 1;
            let next_line_length = self.buffer[next_line_index].len();

            if initial_cursor_horizontal_position > next_line_length {
                self.cursor_position.x = next_line_length as u16;
            }

            self.cursor_position.y += 1;

            if let Ok((_, terminal_height)) = terminal::size() {
                let terminal_height = terminal_height as usize;

                if self.cursor_position.y >= ((self.viewport_start + terminal_height) as u16) {
                    self.viewport_start = (self.cursor_position.y as usize) - terminal_height + 1
                }
            }
        }
    }

    fn save_to_file(&self, filename: &str) -> io::Result<()> {
        let mut file = File::create(filename)?;

        for (i, line) in self.buffer.iter().enumerate() {
            if i < self.buffer.len() - 1 {
                writeln!(file, "{}", line)?;
            } else {
                write!(file, "{}", line)?
            }
        }

        Ok(())
    }

    // TODO: Make this a menu with those options
    fn prompt_if_save(&mut self, stdout: &mut io::Stdout) -> io::Result<bool> {
        disable_raw_mode()?;
        execute!(
            stdout,
            MoveTo(0, self.buffer.len() as u16 + 1),
            terminal::Clear(ClearType::CurrentLine),
            Print("Would you like to save a fule? (y/n/c): ")
        )?;
        stdout.flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        let response = response.trim();

        enable_raw_mode()?;

        match response {
            "y" => {
                if let Some(ref filename) = self.filename {
                    self.save_to_file(filename)?;
                } else {
                    self.prompt_and_save_as(stdout)?;
                }
                Ok(false)
            }
            "n" => Ok(false),
            "c" => Ok(true),
            _ => {
                execute!(
                    stdout,
                    MoveTo(0, self.buffer.len() as u16 + 1),
                    Print("Invalid reponse!")
                )?;

                stdout.flush()?;

                Ok(true)
            }
        }
    }

    fn prompt_and_save_as(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        disable_raw_mode()?;
        execute!(
            stdout,
            MoveTo(0, self.buffer.len() as u16 + 1),
            terminal::Clear(ClearType::CurrentLine),
            Print("Enter filename: ")
        )?;
        stdout.flush()?;

        let mut filename = String::new();
        io::stdin().read_line(&mut filename)?;

        let filename = filename.trim().to_string();

        self.save_to_file(&filename)?;
        self.filename = Some(filename);

        execute!(
            stdout,
            MoveTo(0, self.buffer.len() as u16 + 1),
            Print("File saved!")
        )?;
        stdout.flush()?;
        enable_raw_mode()?;

        Ok(())
    }
}
