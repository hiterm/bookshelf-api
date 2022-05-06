use time::PrimitiveDateTime;

use crate::domain::entity::book::{Book, BookFormat, BookStore, DestructureBook};

#[derive(Debug, Clone)]
pub struct BookDto {
    pub id: String,
    pub title: String,
    pub author_ids: Vec<String>,
    pub isbn: String,
    pub read: bool,
    pub owned: bool,
    pub priority: i32,
    pub format: BookFormat,
    pub store: BookStore,
    pub created_at: PrimitiveDateTime,
    pub updated_at: PrimitiveDateTime,
}

impl From<Book> for BookDto {
    fn from(book: Book) -> Self {
        let DestructureBook {
            id,
            title,
            author_ids,
            isbn,
            read,
            owned,
            priority,
            format,
            store,
            created_at,
            updated_at,
        } = book.destructure();

        Self {
            id: id.to_string(),
            title: title.into_string(),
            author_ids: author_ids
                .into_iter()
                .map(|author_id| author_id.to_string())
                .collect(),
            isbn: isbn.into_string(),
            read: read.to_bool(),
            owned: owned.to_bool(),
            priority: priority.to_i32(),
            format,
            store,
            created_at,
            updated_at,
        }
    }
}
