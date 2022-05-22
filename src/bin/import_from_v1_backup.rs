use std::{collections::HashMap, env, fs::File, io::Read, path::Path};

use serde::{Deserialize, Serialize};

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!("Usage: command <user> <data.json>");
        return Ok(());
    }

    let user = args[1].clone();
    let data_file = args[2].clone();

    let path = Path::new(&data_file);
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    let mut json = String::new();
    file.read_to_string(&mut json)?;
    let backup: BookshelfBackup = serde_json::from_str(&json)?;

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BookshelfBackup {
    books: HashMap<String, Book>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Book {
    title: String,
    authors: Vec<String>,
    isbn: Option<String>,
    read: Option<bool>,
    owned: Option<bool>,
    priority: Option<i32>,
    format: Option<String>,
    store: Option<String>,
    created_at: Option<Time>,
    updated_at: Option<Time>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Time {
    #[serde(rename(serialize = "_seconds"))]
    seconds: u64,
    #[serde(rename(serialize = "_nanoseconds"))]
    nanoseconds: u64,
}
