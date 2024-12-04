mod editor;
mod menu;

use crossterm::{
    cursor::MoveTo,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use editor::Editor;
use menu::Menu;
use std::io::{self, stdout, Write};

fn main() -> io::Result<()> {
    let mut stdout = stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let mut menu = Menu::new();

    loop {
        menu.render(&mut stdout)?;

        if let Some(selected_index) = menu.handle_input(&mut stdout)? {
            match selected_index {
                0 => {
                    let mut editor = Editor::new();

                    loop {
                        if !editor.handle_input(&mut stdout)? {
                            break;
                        }
                    }
                }
                1 => {
                    disable_raw_mode()?;
                    execute!(
                        stdout,
                        MoveTo(0, menu.options.len() as u16 + 1),
                        Print("Enter the file name: ")
                    )?;
                    stdout.flush()?;

                    let mut filename = String::new();
                    io::stdin().read_line(&mut filename)?;

                    let filename = filename.trim();

                    enable_raw_mode()?;

                    let mut editor = Editor::new();
                    if editor.load_from_file(filename).is_ok() {
                        loop {
                            if !editor.handle_input(&mut stdout)? {
                                break;
                            }
                        }
                    } else {
                        execute!(
                            stdout,
                            MoveTo(0, menu.options.len() as u16 + 1),
                            Print("File not found!")
                        )?;
                        stdout.flush()?;
                    }
                }
                2 => break,
                _ => {}
            }
        }
    }

    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    disable_raw_mode()?;
    Ok(())
}
