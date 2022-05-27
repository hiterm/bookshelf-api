use bookshelf_api::{
    common::types::{BookFormat, BookStore},
    domain::entity::{
        author::{Author, AuthorId, AuthorName},
        book::{Book, BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
        user::UserId,
    },
    infrastructure::{
        author_repository::InternalAuthorRepository, book_repository::InternalBookRepository,
    },
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgConnection};
use std::{collections::HashMap, env, fs::File, io::Read, path::Path, time::Duration};
use time::OffsetDateTime;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    // env_logger::init();

    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!("Usage: command <user> <data.json>");
        return Ok(());
    }

    let db_url = fetch_database_url();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_timeout(Duration::from_secs(10))
        .connect(&db_url)
        .await
        .unwrap();
    let mut tx = pool.begin().await?;

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

    let user_id = UserId::new(user)?;
    for (original_id, book) in backup.books {
        let uuid = Uuid::new_v4();

        let author_ids = find_or_create_authors(&user_id, book.authors, &mut tx).await?;

        let format = match BookFormat::try_from(
            book.format.unwrap_or_else(|| "Unknown".to_owned()).as_str(),
        ) {
            Ok(format) => format,
            Err(err) => {
                println!("id: {},\n{}", original_id, err);
                BookFormat::Unknown
            }
        };
        let store = match BookStore::try_from(
            book.store.unwrap_or_else(|| "Unknown".to_owned()).as_str(),
        ) {
            Ok(store) => store,
            Err(err) => {
                println!("id: {},\n{}", original_id, err);
                BookStore::Unknown
            }
        };

        let created_at = book
            .created_at
            .map_or_else(OffsetDateTime::now_utc, |time| {
                OffsetDateTime::from_unix_timestamp(time.seconds)
            });
        let updated_at = book
            .updated_at
            .map_or_else(OffsetDateTime::now_utc, |time| {
                OffsetDateTime::from_unix_timestamp(time.seconds)
            });

        let book = Book::new(
            BookId::new(uuid)?,
            BookTitle::new(book.title)?,
            author_ids,
            Isbn::new(book.isbn.unwrap_or_else(|| "".to_owned()))?,
            ReadFlag::new(book.read.unwrap_or(false)),
            OwnedFlag::new(book.owned.unwrap_or(false)),
            Priority::new(book.priority.unwrap_or(50))?,
            format,
            store,
            created_at,
            updated_at,
        )?;
        InternalBookRepository::create(&user_id, &book, &mut tx).await?;
    }

    // tx.rollback().await?;
    tx.commit().await?;
    println!("finished");

    Ok(())
}

async fn find_or_create_authors(
    user_id: &UserId,
    authors: Vec<String>,
    conn: &mut PgConnection,
) -> anyhow::Result<Vec<AuthorId>> {
    let mut author_ids = vec![];

    for author in authors {
        let author_name = AuthorName::new(author)?;

        let author = find_author_by_name(user_id, &author_name, conn).await?;
        if let Some(author) = author {
            author_ids.push(author.id().to_owned());
            continue;
        }

        let author_id = AuthorId::new(Uuid::new_v4());
        author_ids.push(author_id.clone());
        let author = Author::new(author_id, author_name)?;

        InternalAuthorRepository::create(user_id, &author, conn).await?;
    }

    Ok(author_ids)
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
    #[serde(rename(deserialize = "createdAt"))]
    created_at: Option<Time>,
    #[serde(rename(deserialize = "updatedAt"))]
    updated_at: Option<Time>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Time {
    #[serde(rename(deserialize = "_seconds"))]
    seconds: i64,
    #[serde(rename(deserialize = "_nanoseconds"))]
    nanoseconds: i64,
}
fn fetch_database_url() -> String {
    use std::env::VarError;

    match std::env::var("DATABASE_URL") {
        Ok(s) => s,
        Err(VarError::NotPresent) => panic!("Environment variable DATABASE_URL is required."),
        Err(VarError::NotUnicode(_)) => panic!("Environment variable DATABASE_URL is not unicode."),
    }
}

#[derive(sqlx::FromRow)]
struct AuthorRow {
    id: Uuid,
    name: String,
}

pub async fn find_author_by_name(
    user_id: &UserId,
    author_name: &AuthorName,
    conn: &mut PgConnection,
) -> anyhow::Result<Option<Author>> {
    let row: Option<AuthorRow> =
        sqlx::query_as("SELECT * FROM author WHERE name = $1 AND user_id = $2")
            .bind(author_name.as_str())
            .bind(user_id.as_str())
            .fetch_optional(conn)
            .await?;

    row.map(|row| -> anyhow::Result<Author> {
        let author_id: AuthorId = row.id.into();
        let author_name = AuthorName::new(row.name)?;
        Ok(Author::new(author_id, author_name)?)
    })
    .transpose()
}
