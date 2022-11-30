use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    queue,
    style::Color::{AnsiValue, DarkCyan, Magenta, Yellow},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::env;
use std::io::stdout;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use termimad::{Area, MadSkin, MadView};
use walkdir::WalkDir;
use webbrowser;

// i like me a big a** main file hehe!
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
    let output = Command::new("git")
        .current_dir(get_dir_path())
        .arg("clone")
        .arg(url)
        .output()
        .expect("Failed to execute git clone");

    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    println!("installing latest mdbook");

    let output = Command::new("cargo")
        .arg("install")
        .arg("mdbook")
        .output()
        .expect("Failed to execute cargo install mdbook");

    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    println!("Installed mdbook");

    println!("building the book with mdbook");
    let output = Command::new("mdbook")
        .current_dir(path)
        .arg("build")
        .output()
        .expect("Failed to execute mdbook build");

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

fn get_code_block(flag: String) -> String {
    let mut theflag: &str = &flag.trim();

    if flag.contains("rustdoc") {
        theflag = theflag.strip_prefix("{{#rustdoc_include ../").unwrap();
    } else {
        theflag = theflag.strip_prefix("{{#include ../").unwrap();
    }

    theflag = theflag.strip_suffix("}}").unwrap();

    if theflag.contains(":") {
        (theflag, _) = theflag.split_once(":").unwrap();
    }

    let code_path = get_book_path() + theflag;

    let code = std::fs::read_to_string(code_path).unwrap();

    code
}

fn parse_listings(section: &mut Section) {
    let content = &section.content.as_bytes();
    let mut new_content: Vec<u8> = Vec::new();

    let mut flag: Vec<u8> = Vec::new();

    let mut writing: bool = true;

    for (index, character) in content.iter().enumerate() {
        if index + 2 < content.len() && content[index + 1] == b'{' && content[index + 2] == b'{' {
            writing = false;
        }

        if index > 0 && content[index - 1] == b'}' && content[index - 2] == b'}' {
            let new_flag = std::str::from_utf8(&flag).unwrap().to_string();

            let code_block = get_code_block(new_flag);
            let code = code_block.as_bytes();

            new_content.push(b'\n');

            for item in code {
                new_content.push(*item)
            }

            flag = Vec::new();

            writing = true;
        }

        if !writing {
            flag.push(*character);
        }

        if writing {
            new_content.push(*character);
        }
    }

    section.content = std::str::from_utf8(&new_content).unwrap().to_string();
}

fn search_book(words: &Vec<String>) -> Vec<Section> {
    let folder_path = get_book_path() + "src";
    let files = get_files(&folder_path);

    let mut final_sections = Vec::new();

    for file in files {
        let sections: Vec<Section> = search_page(&file, &words);

        for section in sections {
            if section.mentions > 1 {
                final_sections.push(section)
            }
        }
    }

    for (_index, section) in final_sections.iter_mut().enumerate() {
        parse_listings(section);
    }

    final_sections
}

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
        if content.len() > index + 2 && content[index + 1] == b'#' && content[index + 2] == b'#' {
            let section = std::str::from_utf8(&current_section).unwrap().to_string();
            sections.push(section);
            current_section = Vec::new();
        }
        current_section.push(*character);

        if index == content.len() - 1 {
            let section = std::str::from_utf8(&current_section).unwrap().to_string();
            sections.push(section);
            current_section = Vec::new();
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

    // reward query mention in heading
    for mut valid_section in valid_sections.as_mut_slice() {
        let mut heading: Vec<u8> = Vec::new();
        let mut writing: bool = false;

        let content = valid_section.content.as_bytes();

        for (index, character) in content.iter().enumerate() {
            if content[index] == b'#' {
                writing = true;
            }

            if content[index] == b'\n' {
                break;
            }

            if writing {
                heading.push(*character);
            }
        }

        let heading = std::str::from_utf8(&heading).unwrap().to_string();
        let heading = heading.to_lowercase();

        for query in queries {
            if heading.contains(query) {
                valid_section.mentions += 20
            }
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
                valid_section.mentions += 4
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

fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);

    verify_dir();
    verify_book();

    if args.len() == 0 {
        open_book()
    } else {
        println!("searching book for {:?}", &args);

        let mut final_sections = search_book(&args);

        final_sections.sort_by_key(|i| i.mentions);
        final_sections.reverse();

        println!("Found {} results", final_sections.len());

        if final_sections.len() == 0 {
            return;
        }

        let mut cursor: u32 = 0;
        loop {
            let content = &final_sections[cursor as usize].content;

            let debug = format!("Debug: Result {} of {}", cursor + 1, final_sections.len())
                + &format!(" | Scored {} pts", final_sections[cursor as usize].mentions)
                + &format!(" for {:?}", args)
                + &format!(" | Change results with <-- and --> arrow keys or H and L")
                + &format!(" | Close The Book with Q ")
                + &format!(" | Open in browser with O ");

            let text = content.clone() + "\n" + r#"```text"# + "\n" + &debug + "\n" + r#"```"#;

            cursor = print_markdown(&final_sections, &text, cursor);
        }
    }
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

                KeyCode::Char('c') => {
                    terminal::disable_raw_mode().unwrap();
                    queue!(writer, LeaveAlternateScreen).unwrap();
                    writer.flush().unwrap();

                    std::process::exit(0x0100);
                }
                KeyCode::Char('q') => {
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

//.?
//             ⣠⣤⣤⣤⣤⣤⣶⣦⣤⣄⡀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⢀⣴⣿⡿⠛⠉⠙⠛⠛⠛⠛⠻⢿⣿⣷⣤⡀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⣼⣿⠋⠀⠀⠀⠀⠀⠀⠀⢀⣀⣀⠈⢻⣿⣿⡄⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⣸⣿⡏⠀⠀⠀⣠⣶⣾⣿⣿⣿⠿⠿⠿⢿⣿⣿⣿⣄⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⣿⣿⠁⠀⠀⢰⣿⣿⣯⠁⠀⠀⠀⠀⠀⠀⠀⠈⠙⢿⣷⡄⠀
//⠀⠀⣀⣤⣴⣶⣶⣿⡟⠀⠀⠀⢸⣿⣿⣿⣆🔴⠀⠀⠀⠀🔴⠀⠀⣿⣷⠀
//⠀⢰⣿⡟⠋⠉⣹⣿⡇⠀⠀⠀⠘⣿⣿⣿⣿⣷⣦⣤⣤⣤⣶⣶⣶⣶⣿⣿⣿⠀
//⠀⢸⣿⡇⠀⠀⣿⣿⡇⠀⠀⠀⠀⠹⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠃⠀
//⠀⣸⣿⡇⠀⠀⣿⣿⡇⠀⠀⠀⠀⠀⠉⠻⠿⣿⣿⣿⣿⡿⠿⠿⠛⢻⣿⡇⠀⠀
//⠀⣿⣿⠁⠀⠀⣿⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⣧⠀⠀
//⠀⣿⣿⠀⠀⠀⣿⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⣿⠀⠀
//⠀⣿⣿⠀⠀⠀⣿⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⣿⠀⠀
//⠀⢿⣿⡆⠀⠀⣿⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⡇⠀⠀
//⠀⠸⣿⣧⡀⠀⣿⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⠃⠀⠀
//⠀⠀⠛⢿⣿⣿⣿⣿⣇⠀⠀⠀⠀⠀⣰⣿⣿⣷⣶⣶⣶⣶⠶⠀⢠⣿⣿⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⣿⣿⠀⠀⠀⠀⠀⣿⣿⡇⠀⣽⣿⡏⠁⠀⠀⢸⣿⡇⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⣿⣿⠀⠀⠀⠀⠀⣿⣿⡇⠀⢹⣿⡆⠀⠀⠀⣸⣿⠇⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⢿⣿⣦⣄⣀⣠⣴⣿⣿⠁⠀⠈⠻⣿⣿⣿⣿⡿⠏⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠈⠛⠻⠿⠿⠿⠿⠋⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//?.

//.?
//⬜️⬜️🟥🟥🟥⬜️⬜️
//⬜️🟥🟥🟥🟥🟥⬜️
//🟥🟥⬛️⬛️⬛️🟥⬜️
//🟥🟥🟥🟥🟥🟥⬜️
//🟥🟥🟥🟥🟥🟥⬜️
//⬜️🟥🟥⬜️🟥🟥⬜️
//⬜️🟥🟥⬜️🟥🟥⬜️    .

//.? ⠀
//      ⠀⠀⠀⠀⠀⢀⣀⣀⣴⣆⣠⣤⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⣻⣿⣯⣘⠹⣧⣤⡀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠛⠿⢿⣿⣷⣾⣯⠉⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣾⣿⠜⣿⡍⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣸⣿⠁⠀⠘⣿⣆⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⣿⡟⠃⡄⠀⠘⢿⣆⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣾⣿⣁⣋⣈ ⣤⣮⣿⣧⡀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⣤⣤⣤⣤⣤⣶⣦⣤⣄⡀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⢀⣴⣿⡿⠛⠉⠙⠛⠛⠛⠛⠻⢿⣿⣷⣤⡀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⣼⣿⠋⠀⠀⠀⠀⠀⠀⠀⢀⣀⣀⠈⢻⣿⣿⡄⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⣸⣿⡏⠀⠀⠀⣠⣶⣾⣿⣿⣿⠿⠿⠿⢿⣿⣿⣿⣄⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⣿⣿⠁⠀⠀⢰⣿⣿⣯⠁⠀⠀⠀⠀⠀⠀⠀⠈⠙⢿⣷⡄⠀
//⠀⠀⣀⣤⣴⣶⣶⣿⡟⠀⠀⠀⢸⣿⣿⣿⣆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣷⠀
//⠀⢰⣿⡟⠋⠉⣹⣿⡇⠀⠀⠀⠘⣿⣿⣿⣿⣷⣦⣤⣤⣤⣶⣶⣶⣶⣿⣿⣿⠀
//⠀⢸⣿⡇⠀⠀⣿⣿⡇⠀⠀⠀⠀⠹⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠃⠀
//⠀⣸⣿⡇⠀⠀⣿⣿⡇⠀⠀⠀⠀⠀⠉⠻⠿⣿⣿⣿⣿⡿⠿⠿⠛⢻⣿⡇⠀⠀
//⠀⣿⣿⠁⠀⠀⣿⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⣧⠀⠀
//⠀⣿⣿⠀⠀⠀⣿⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⣿⠀⠀
//⠀⣿⣿⠀⠀⠀⣿⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⣿⠀⠀
//⠀⢿⣿⡆⠀⠀⣿⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⡇⠀⠀
//⠀⠸⣿⣧⡀⠀⣿⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⠃⠀⠀
//⠀⠀⠛⢿⣿⣿⣿⣿⣇⠀⠀⠀⠀⠀⣰⣿⣿⣷⣶⣶⣶⣶⠶⠀⢠⣿⣿⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⣿⣿⠀⠀⠀⠀⠀⣿⣿⡇⠀⣽⣿⡏⠁⠀⠀⢸⣿⡇⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⣿⣿⠀⠀⠀⠀⠀⣿⣿⡇⠀⢹⣿⡆⠀⠀⠀⣸⣿⠇⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⢿⣿⣦⣄⣀⣠⣴⣿⣿⠁⠀⠈⠻⣿⣿⣿⣿⡿⠏⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠈⠛⠻⠿⠿⠿⠿⠋⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//?.
