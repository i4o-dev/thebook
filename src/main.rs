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

static VERSION: &str = "0.3.0";
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

// Sections from the book
#[derive(Debug)]
pub struct Section {
    content: String,
    mentions: u32,
    link: String,
}

fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);

    verify_dir(); // verify that the book directory exists else create it
    verify_book(); // verify that the book is downloaded and in the right directory. else, download it

    if args.len() == 0 {
        // open the book in the browser if no arguments were given
        let index_path = get_book_path() + "book/index.html";
        open_book(&index_path)
    } else {
        let flag = args[0].as_str();

        match flag {
            "--reset" => reset_book(), // delete the book and download a fresh copy
            "-h" | "--help" => {
                println!("{}", HELP);
            }

            "-v" | "--version" => {
                println!("thebook {}", VERSION);
            }
            _ => {
                println!("searching book for {:?}", args);

                // search the book for the query and return sections sorted by mentions
                let mut final_sections = search_book(&args);
                final_sections.sort_by_key(|i| i.mentions);
                final_sections.reverse();

                println!("Found {} results", final_sections.len());

                if final_sections.len() == 0 {
                    return; // exit if no results were found
                }

                // renders the sections (markdown) in the terminal ui
                print_markdown(&final_sections);
            }
        }
    }
}

// searches through the book and returns sections that mention the search query
fn search_book(words: &Vec<String>) -> Vec<Section> {
    let folder_path = get_book_path() + "src"; // get a valid path to the book
    let files = get_files(&folder_path); // get a list of all pages in the book

    let mut final_sections = Vec::new();

    // loop through every page in the book, split each page into sections, and
    // return only sections that mention the search query in their title or body
    for file in files {
        let sections: Vec<Section> = search_page(&file, &words); // returns the page split up into sections

        // eliminate sections that do not mention the search query
        for section in sections {
            if section.mentions > 1 {
                final_sections.push(section)
            }
        }
    }

    // certain sections have code examples which are stored in separate files
    // this loop mutates all sections and includes code examples in sections that need them
    for (_index, section) in final_sections.iter_mut().enumerate() {
        parse_listings(section);
    }

    final_sections
}

// includes a specific code example
fn get_code_block(flag: &String) -> String {
    let mut theflag: &str = &flag.trim();

    // turns the flag into a valid url
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

// mutates all sections to add code examples to sections that need them
fn parse_listings(section: &mut Section) {
    let content = &section.content.as_bytes();
    let mut new_content: Vec<u8> = Vec::new();

    let mut flag: Vec<u8> = Vec::new();

    let mut writing1: bool = true;
    let mut writing2: bool = true;

    // uses the IMPOSTER DETECTION ALGORITHM to identify code samples uwu
    for (index, character) in content.iter().enumerate() {
        if index + 2 < content.len() && content[index + 1] == b'{' && content[index + 2] == b'{' {
            writing1 = false;
        }

        if index > 0 && content[index - 1] == b'}' && content[index - 2] == b'}' {
            let new_flag = std::str::from_utf8(&flag).unwrap().to_string();

            let code_block = get_code_block(&new_flag);
            let code = code_block.as_bytes();

            new_content.push(b'\n');

            for item in code {
                new_content.push(*item)
            }

            flag = Vec::new();

            writing1 = true;
        }

        if !writing1 {
            flag.push(*character);
        }

        // must be below the above condition for eliminating html tags
        if index < content.len() && content[index] == b'<' {
            writing2 = false
        }
        if index > 1 && content[index - 1] == b'>' {
            writing2 = true
        }

        if writing1 && writing2 {
            new_content.push(*character);
        }
    }

    section.content = std::str::from_utf8(&new_content).unwrap().to_string();
}

// splits a page (source file) into more manageable and searchable sections
// and rewards each section by how many times they mention the search query
fn search_page(page_path: &String, queries: &Vec<String>) -> Vec<Section> {
    let page_content = std::fs::read_to_string(&page_path).unwrap();

    let mut mentions = 0;

    // since queries can be multiple, reward pages that mentions each query
    for query in queries {
        if page_content.contains(query) {
            mentions += 1;
        }
    }

    if mentions == 0 {
        // return an empty vector if the page does not mention any query
        return Vec::new();
    }

    // splits the page into sections
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

    // reward query mention in the section's title
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

// deletes the local copy of thebook and downloads a fresh copy
fn reset_book() {
    println!("Downloading latest copy of The Book");

    let book_path = get_book_path();

    std::fs::remove_dir_all(book_path).unwrap(); // deletes the local copy

    verify_book(); // verify_book() downloads the book if it does not exist

    println!("The Book has been reset");
}

// opens the link to a local html/md file in the default browser
fn open_book(link: &String) {
    println!("opening book");

    // if the link specifies a html file, use it directly
    // else if the link specifies a markdown file, find the corresponding
    // html file and open it
    if link.ends_with(".html") {
        webbrowser::open(&link).unwrap();
    } else {
        let link = link.replace("src", "book");
        let link = link.replace(".md", ".html");

        webbrowser::open(&link).unwrap();
    }
}

// downloads the book's source files from the official github repository and builds it with mdbook
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
