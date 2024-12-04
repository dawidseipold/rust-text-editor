use std::io;

use crossterm::{
    cursor::MoveTo,
    event::{read, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    style::Print,
    terminal::{self, ClearType},
};

pub struct Menu {
    pub selected_index: usize,
    pub options: Vec<String>,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            options: vec![
                String::from("Create a new text"),
                String::from("Edit an existing text"),
                String::from("Exit"),
            ],
        }
    }

    pub fn render(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(stdout, terminal::Clear(ClearType::All))?;

        for (i, option) in self.options.iter().enumerate() {
            let prefix = if i == self.selected_index { ">" } else { " " };

            execute!(
                stdout,
                MoveTo(0, i as u16),
                Print(format!("{} {}", prefix, option))
            )?;
        }

        Ok(())
    }

    pub fn handle_input(&mut self, stdout: &mut io::Stdout) -> io::Result<Option<usize>> {
        if let Event::Key(KeyEvent { code, kind, .. }) = read()? {
            if kind == KeyEventKind::Press {
                match code {
                    KeyCode::Up => {
                        if self.selected_index > 0 {
                            self.selected_index -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if self.selected_index < self.options.len() - 1 {
                            self.selected_index += 1;
                        }
                    }
                    KeyCode::Enter => return Ok(Some(self.selected_index)),
                    _ => {}
                }
            }
        }

        self.render(stdout)?;

        Ok(None)
    }
}
