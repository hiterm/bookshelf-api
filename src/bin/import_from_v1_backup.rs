use std::{collections::HashMap, env, fs::File, io::Read, path::Path};

use bookshelf_api::domain::{
    entity::{
        author::AuthorId,
        book::{
            Book, BookFormat, BookId, BookStore, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag,
        },
    },
    error::DomainError,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

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

    for (id, book) in backup.books {
        let uuid = Uuid::new_v4();

        // TODO: author

        let author_ids = find_or_create_authors(book.authors);
        let created_at = book.created_at.map_or_else(
            || OffsetDateTime::now_utc(),
            |time| OffsetDateTime::from_unix_timestamp(time.seconds),
        );
        let updated_at = book.updated_at.map_or_else(
            || OffsetDateTime::now_utc(),
            |time| OffsetDateTime::from_unix_timestamp(time.seconds),
        );

        let book = Book::new(
            BookId::new(uuid)?,
            BookTitle::new(book.title)?,
            author_ids,
            Isbn::new(book.isbn.unwrap_or_else(|| "".to_owned()))?,
            ReadFlag::new(book.read.unwrap_or(false)),
            OwnedFlag::new(book.owned.unwrap_or(false)),
            Priority::new(book.priority.unwrap_or(50))?,
            BookFormat::try_from(book.format.unwrap_or_else(|| "".to_owned()).as_str())?,
            BookStore::try_from(book.store.unwrap_or_else(|| "".to_owned()).as_str())?,
            created_at,
            updated_at,
        )?;
    }

    Ok(())
}

fn find_or_create_authors(authors: Vec<String>) -> Vec<AuthorId> {
    todo!()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BookshelfBackup {
    books: HashMap<String, BackupBook>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackupBook {
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
    seconds: i64,
    #[serde(rename(serialize = "_nanoseconds"))]
    nanoseconds: i64,
}
