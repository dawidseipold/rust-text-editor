use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::Print,
    terminal::{self, ClearType},
};
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::Clear(ClearType::All))?;

    let mut buffer = String::new();

    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind != KeyEventKind::Press {
                    continue;
                }

                match key_event.code {
                    KeyCode::Char(c) => {
                        buffer.push(c);
                        execute!(
                            stdout,
                            cursor::MoveTo(0, 0),
                            terminal::Clear(ClearType::CurrentLine),
                            Print(&buffer)
                        )?;
                        stdout.flush()?;
                    }
                    KeyCode::Backspace => {
                        buffer.pop();
                        execute!(
                            stdout,
                            cursor::MoveTo(0, 0),
                            terminal::Clear(ClearType::CurrentLine),
                            Print(&buffer)
                        )?;
                        stdout.flush()?;
                    }
                    KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }

    terminal::disable_raw_mode()?;
    execute!(stdout, cursor::Show)?;
    Ok(())
}
