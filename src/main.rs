use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    queue,
    style::Color::{AnsiValue, DarkCyan, Magenta, Yellow},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use git2::Repository;
use std::env;
use std::io::stdout;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use termimad::{Area, MadSkin, MadView};
use walkdir::WalkDir;
use webbrowser;

fn get_dir_path() -> String {
    let path = dirs::home_dir().unwrap().to_str().unwrap().to_string() + "/.thebook/";
    path
}

fn get_book_path() -> String {
    let path = get_dir_path() + "book/";
    path
}

fn verify_dir() {
    let path = get_dir_path();
    if !dir_exists(&path) {
        create_dir(&path);
    }
}

fn dir_exists(path: &String) -> bool {
    if Path::new(&path).exists() {
        true
    } else {
        println!("path does not exist");
        false
    }
}

fn create_dir(path: &String) {
    std::fs::create_dir(path).unwrap();
}

fn verify_book() {
    let path = get_book_path();
    if !book_exists(&path) {
        fetch_book(&path);
    }
}

fn book_exists(path: &String) -> bool {
    if Path::new(&path).exists() {
        true
    } else {
        println!("book does not exist");
        false
    }
}

fn fetch_book(path: &String) {
    let url = "https://github.com/rust-lang/book";
    println!("Cloning the book from: {}", url);

    let _ = match Repository::clone(url, path) {
        Ok(_) => {
            println!("cloned the book from github")
        }
        Err(e) => panic!("failed to clone: {}", e),
    };

    println!("installing latest mdbook");

    let output = Command::new("cargo")
        .arg("install")
        .arg("mdbook")
        .output()
        .expect("Failed to execute command");

    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    println!("Installed mdbook");

    println!("building the book with mdbook");
    let output = Command::new("mdbook")
        .current_dir(path)
        .arg("build")
        .output()
        .expect("Failed to execute command");

    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    println!("build complete")
}

fn open_book() {
    println!("opening book");
    let index_path = get_book_path() + "book/index.html";

    webbrowser::open(&index_path).unwrap();
}

fn get_files(folder_path: &String) -> Vec<String> {
    let mut files: Vec<String> = Vec::new();

    for entry in WalkDir::new(folder_path) {
        let entry = entry.unwrap();

        let path = entry.path().to_str().unwrap();

        if path.ends_with(".md") {
            files.push(path.to_string());
        }
    }

    files
}

// returns a list of markdown strings
fn search_book(words: &Vec<String>) -> Vec<Section> {
    let folder_path = get_book_path() + "src";
    let files = get_files(&folder_path);

    let mut final_results = Vec::new();

    for file in files {
        let sections: Vec<Section> = search_page(&file, &words);

        for section in sections {
            if section.mentions > 1 {
                final_results.push(section)
            }
        }
    }

    final_results
}

// returns how many times all queries were mentioned in this page
// TODO: introduce bias by checking titles/headings and rewarding pages that have queries in their headings with 20 points
// TODO: mentions of queries in the file path should also add bias of 20 points per query mention
// TODO: introduce FAQ bias where certain common questions reward certain pages with points, such as 'what is ownership' should reward chapter 4 page 1 with 40 points
fn search_page(page_path: &String, queries: &Vec<String>) -> Vec<Section> {
    let page_content = std::fs::read_to_string(&page_path).unwrap();

    let mut mentions = 0;

    for query in queries {
        if page_content.contains(query) {
            mentions += 1;
        }
    }

    if mentions == 0 {
        return Vec::new();
    }

    let mut sections: Vec<String> = Vec::new();
    let mut current_section: Vec<u8> = Vec::new();

    let content = page_content.as_bytes();
    for (index, character) in content.iter().enumerate() {
        if content[index] == 35 && content[index + 1] == 35 {
            let section = std::str::from_utf8(&current_section).unwrap().to_string();
            sections.push(section);
            current_section = Vec::new();
        } else {
            current_section.push(*character);
        }
    }

    let mut valid_sections: Vec<Section> = Vec::new();

    for section in sections {
        let new_section = Section {
            content: section,
            mentions,
        };

        if new_section.content.len() > 5 {
            valid_sections.push(new_section);
        }
    }

    // reward query mention in file path
    for mut valid_section in valid_sections.as_mut_slice() {
        for query in queries {
            if page_path.contains(query) {
                valid_section.mentions += 20
            }
        }
    }

    // reward query mention in section content
    for mut valid_section in valid_sections.as_mut_slice() {
        for query in queries {
            if valid_section.content.contains(query) {
                valid_section.mentions += 2
            }
        }
    }

    valid_sections
}

#[derive(Debug)]
struct Section {
    content: String,
    mentions: u32,
}

fn print_markdown(results: &Vec<Section>, text: &String, cursor: u32) -> u32 {
    let length = results.len() as u32;
    let mut new_cursor = cursor;

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
        match event::read() {
            Ok(Event::Key(KeyEvent { code, .. })) => match code {
                KeyCode::Up => view.try_scroll_lines(-1),
                KeyCode::Down => view.try_scroll_lines(1),
                KeyCode::PageUp => view.try_scroll_pages(-1),
                KeyCode::PageDown => view.try_scroll_pages(1),
                KeyCode::Char('d') => {
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

                KeyCode::Char('c') => {
                    terminal::disable_raw_mode().unwrap();
                    queue!(writer, LeaveAlternateScreen).unwrap();
                    writer.flush().unwrap();

                    std::process::exit(0x0100);
                }

                KeyCode::Char('o') => open_book(),

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

fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);

    verify_dir();
    verify_book();

    if args.len() == 0 {
        open_book()
    } else {
        println!("searching book for {:?}", &args);

        let mut final_results = search_book(&args);

        final_results.sort_by_key(|i| i.mentions);
        final_results.reverse();

        println!("Found {} results", final_results.len());

        let mut cursor: u32 = 0;
        loop {
            let content = &final_results[cursor as usize].content;

            let debug = format!("Result {} of {}", cursor + 1, final_results.len())
                + &format!(" | Scored {} pts", final_results[cursor as usize].mentions)
                + &format!(" for {:?}", args)
                + &format!(" | Change results with <-- and --> arrow keys or A and D")
                + &format!(" | Close The Book with C ");

            let text = content.clone() + &debug;

            cursor = print_markdown(&final_results, &text, cursor);
        }
    }
}
