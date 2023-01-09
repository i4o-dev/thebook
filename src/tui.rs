use crate::open_book;
use crate::Section;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    queue,
    style::Color::{AnsiValue, DarkCyan, Magenta, Yellow},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::stdout;
use std::io::Write;
use termimad::{Area, MadSkin, MadView};

pub fn print_markdown(results: &Vec<Section>, text: &String, cursor: u32) -> u32 {
    let length = results.len() as u32;
    let mut new_cursor = cursor;
    let link = &results[cursor as usize].link;

    let mut skin = MadSkin::default();
    skin.set_headers_fg(AnsiValue(178));
    skin.bold.set_fg(Yellow);
    skin.italic.set_fg(Magenta);
    skin.scrollbar.thumb.set_fg(AnsiValue(178));
    skin.code_block.set_fg(DarkCyan);
    skin.inline_code.set_fg(DarkCyan);

    let area = Area::full_screen();
    let mut view = MadView::from(text.to_owned(), area, skin);

    let mut writer = stdout(); // we could also have used stderr
    queue!(writer, EnterAlternateScreen).unwrap();
    terminal::enable_raw_mode().unwrap();

    loop {
        view.write_on(&mut writer).unwrap();
        writer.flush().unwrap();

        let mut quit = || {
            terminal::disable_raw_mode().unwrap();
            queue!(writer, LeaveAlternateScreen).unwrap();
            writer.flush().unwrap();

            std::process::exit(0x0100);
        };

        match event::read() {
            Ok(Event::Key(KeyEvent { code, .. })) => match code {
                KeyCode::Up => view.try_scroll_lines(-1),
                KeyCode::Down => view.try_scroll_lines(1),
                KeyCode::PageUp => view.try_scroll_pages(-1),
                KeyCode::PageDown => view.try_scroll_pages(1),

                KeyCode::Char('j') => view.try_scroll_lines(1),
                KeyCode::Char('k') => view.try_scroll_lines(-1),

                KeyCode::Char('d') => {
                    if new_cursor + 1 < length {
                        new_cursor += 1;
                        break;
                    } else {
                        break;
                    }
                }
                KeyCode::Char('l') => {
                    if new_cursor + 1 < length {
                        new_cursor += 1;
                        break;
                    } else {
                        break;
                    }
                }

                KeyCode::Char('a') => {
                    if new_cursor != 0 {
                        new_cursor -= 1;
                        break;
                    } else {
                        break;
                    }
                }
                KeyCode::Char('h') => {
                    if new_cursor != 0 {
                        new_cursor -= 1;
                        break;
                    } else {
                        break;
                    }
                }

                KeyCode::Right => {
                    if new_cursor + 1 < length {
                        new_cursor += 1;
                        break;
                    } else {
                        break;
                    }
                }
                KeyCode::Left => {
                    if new_cursor != 0 {
                        new_cursor -= 1;
                        break;
                    } else {
                        break;
                    }
                }

                KeyCode::Char('c') => quit(),
                KeyCode::Char('q') => quit(),
                KeyCode::Esc => quit(),

                KeyCode::Char('o') => open_book(link),

                _ => {
                    println!("invalid key!")
                }
            },
            Ok(Event::Resize(..)) => {
                queue!(writer, Clear(ClearType::All)).unwrap();
                view.resize(&Area::full_screen());
            }
            _ => {}
        }
    }

    terminal::disable_raw_mode().unwrap();
    queue!(writer, LeaveAlternateScreen).unwrap();
    writer.flush().unwrap();

    new_cursor
}
