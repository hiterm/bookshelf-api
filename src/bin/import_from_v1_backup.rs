use std::{env, fs::File, io::Read, path::Path};

use serde::{Deserialize, Serialize};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("Usage: command <user> <data.json>");
        return;
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
    match file.read_to_string(&mut json) {
        Err(why) => panic!("couldn't read {}: {}", display, why),
        Ok(_) => print!("{} contains:\n{}", display, json),
    }
}

#[derive(Serialize, Deserialize)]
struct BookshelfBackup {}
