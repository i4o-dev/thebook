use std::{
    env,
    io::{self, Write},
    process::Command,
};
use webbrowser;

mod fs_utils;
mod tui;

use fs_utils::*;
use tui::*;

static VERSION: &str = "0.2.3";
static HELP: &str = r#"
TheBook (Read and Search The Rust Book from the terminal)

Usage:
  thebook
  thebook <your search query>
  thebook -h | --help
  thebook -v | --version

Options:
  -h --help      Show this screen.
  -v --version   Show version.
  --reset        Download latest Book
    "#;

#[derive(Debug)]
pub struct Section {
    content: String,
    mentions: u32,
    link: String,
}

fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);

    verify_dir();
    verify_book();

    if args.len() == 0 {
        let index_path = get_book_path() + "book/index.html";
        open_book(&index_path)
    } else {
        let flag = args[0].as_str();

        match flag {
            "--reset" => reset_book(),
            "-h" | "--help" => {
                println!("{}", HELP);
            }

            "-v" | "--version" => {
                println!("thebook {}", VERSION);
            }
            _ => {
                println!("searching book for {:?}", args);

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
                        + &format!(" | Change results with ← and → arrow keys or H and L")
                        + &format!(" | Scroll up and down with ↑ and ↓ arrow keys or J and K")
                        + &format!(" | Open this page in web browser with O ")
                        + &format!(" | Quit with Q ");

                    let text =
                        content.clone() + "\n" + r#"```text"# + "\n" + &debug + "\n" + r#"```"#;

                    cursor = print_markdown(&final_sections, &text, cursor);
                }
            }
        }
    }
}

fn reset_book() {
    println!("Downloading latest copy of The Book");

    let book_path = get_book_path();

    std::fs::remove_dir_all(book_path).unwrap();

    verify_book();

    println!("The Book has been reset");
}

fn open_book(link: &String) {
    println!("opening book");

    if link.ends_with(".html") {
        webbrowser::open(&link).unwrap();
    } else {
        let link = link.replace("src", "book");
        let link = link.replace(".md", ".html");

        webbrowser::open(&link).unwrap();
    }
}

pub fn fetch_book(path: &String) {
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

    println!("Installing latest mdbook");

    let output = Command::new("cargo")
        .arg("install")
        .arg("mdbook")
        .output()
        .expect("Failed to execute cargo install mdbook");

    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    println!("Installed mdbook");

    println!("Building the book with mdbook");
    let output = Command::new("mdbook")
        .current_dir(path)
        .arg("build")
        .output()
        .expect("Failed to execute mdbook build");

    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    println!("Build complete");

    println!("The Book has been installed");
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
            link: page_path.clone(),
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
