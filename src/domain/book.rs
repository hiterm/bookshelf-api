use time::PrimitiveDateTime;

pub struct BookshelfUser {
    id: String,
    sub: String,
}

pub struct Book {
    id: String,
    user: BookshelfUser,
    authors: Vec<Author>,
    isbn: Option<String>,
    read: bool,
    owned: bool,
    priority: u32,
    format: Option<BookFormat>,
    store: Option<BookStore>,
    created_at: PrimitiveDateTime,
    updated_at: PrimitiveDateTime,
}

pub enum BookFormat {
    EBook,
    Printed,
}

pub enum BookStore {
    Kindle,
}

pub struct Author {
    id: String,
    name: String,
}
