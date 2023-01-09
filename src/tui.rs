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

pub fn print_markdown(results: &Vec<Section>) {
    let length = results.len() as u32;

    let mut writer = stdout();
    queue!(writer, EnterAlternateScreen).unwrap();
    terminal::enable_raw_mode().unwrap();

    let mut cursor = 0;
    let mut scroll: usize = 0;

    loop {
        let mut new_cursor = cursor;
        let link = &results[cursor as usize].link;

        let content = &results[cursor as usize].content;

        // debug message printed under every page
        let debug = format!("Debug: Result {} of {}", cursor + 1, results.len())
            + &format!(" | Scored {} pts", results[cursor as usize].mentions)
            + &format!(" | Change results with ← and → arrow keys or H and L")
            + &format!(" | Scroll up and down with ↑ and ↓ arrow keys or J and K")
            + &format!(" | Open this page in web browser with O ")
            + &format!(" | Quit with Q ");

        let text = content.clone() + "\n" + r#"```text"# + "\n" + &debug + "\n" + r#"```"#;

        let mut skin = MadSkin::default();
        skin.set_headers_fg(AnsiValue(178));
        skin.bold.set_fg(Yellow);
        skin.italic.set_fg(Magenta);
        skin.scrollbar.thumb.set_fg(AnsiValue(178));
        skin.code_block.set_fg(DarkCyan);
        skin.inline_code.set_fg(DarkCyan);

        let area = Area::full_screen();
        let mut view = MadView::from(text.to_owned(), area, skin);
        view.try_scroll_lines(scroll as i32);

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
                KeyCode::Up | KeyCode::Char('k') | KeyCode::PageUp => view.try_scroll_lines(-1),
                KeyCode::Down | KeyCode::Char('j') | KeyCode::PageDown => view.try_scroll_lines(1),

                KeyCode::Char('d') | KeyCode::Char('l') | KeyCode::Right => {
                    if new_cursor + 1 < length {
                        new_cursor += 1;
                        // break;
                    } else {
                        //  break;
                    }
                }

                KeyCode::Char('a') | KeyCode::Char('h') | KeyCode::Left => {
                    if new_cursor != 0 {
                        new_cursor -= 1;
                        //  break;
                    } else {
                        //   break;
                    }
                }

                KeyCode::Char('c') | KeyCode::Char('q') | KeyCode::Esc => quit(),

                KeyCode::Char('o') => open_book(link),

                _ => {}
            },
            Ok(Event::Resize(..)) => {
                queue!(writer, Clear(ClearType::All)).unwrap();
                view.resize(&Area::full_screen());
            }
            _ => {}
        }

        scroll = view.scroll;

        cursor = new_cursor;
    }
}
