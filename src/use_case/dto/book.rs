use time::PrimitiveDateTime;

#[derive(Debug, Clone)]
pub struct BookDto {
    pub id: String,
    pub title: String,
    pub author_ids: Vec<String>,
    pub isbn: String,
    pub read: bool,
    pub owned: bool,
    pub priority: i32,
    pub format: String,
    pub store: String,
    pub created_at: PrimitiveDateTime,
    pub updated_at: PrimitiveDateTime,
}
