use crate::fetch_book;
use std::path::Path;
use walkdir::WalkDir;

pub fn get_files(folder_path: &String) -> Vec<String> {
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

pub fn get_dir_path() -> String {
    let path = dirs::home_dir().unwrap().to_str().unwrap().to_string() + "/thebook/";
    path
}

pub fn get_book_path() -> String {
    let path = get_dir_path() + "book/";
    path
}

pub fn verify_dir() {
    let path = get_dir_path();
    if !dir_exists(&path) {
        create_dir(&path);
    }
}

pub fn dir_exists(path: &String) -> bool {
    if Path::new(&path).exists() {
        true
    } else {
        println!("Path does not exist");
        false
    }
}

pub fn create_dir(path: &String) {
    std::fs::create_dir(path).unwrap();
}

pub fn verify_book() {
    let path = get_book_path();
    if !book_exists(&path) {
        fetch_book(&path);
    }
}

pub fn book_exists(path: &String) -> bool {
    if Path::new(&path).exists() {
        true
    } else {
        println!("The Book does not exist");
        false
    }
}
