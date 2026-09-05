use async_graphql::dataloader::DataLoader;
use async_graphql::{ComplexObject, Context, Enum, Json, Result};
use async_graphql::{ID, InputObject, SimpleObject};
use serde_json::Value;
use time::{Date, OffsetDateTime};

use crate::common::types::{BookFormat as CommonBookFormat, BookStore as CommonBookStore};
use crate::dependency_injection::{AQ, BQ, HQ};
use crate::presentation::extractor::claims::Claims;
use crate::use_case::dto::author::{AuthorDto, CreateAuthorDto, UpdateAuthorDto};
use crate::use_case::dto::book::{
    BookDto, CreateBookDto, ImportAuthorPreviewDto, ImportAuthorStatus as ImportAuthorStatusDto,
    ImportBookEntryDto, ImportBookPreviewDto, ImportBooksPreviewDto, UpdateBookDto,
};
use crate::use_case::dto::history::{
    AuthorOperationChangeDto, AuthorRevisionDto, BookOperationChangeDto, BookRevisionDto,
    OperationDto,
};
use crate::use_case::traits::history::HistoryQueryUseCase;

use super::loader::{
    AuthorChangesByOperationLoader, AuthorLoader, BookChangesByOperationLoader, BooksByAuthorLoader,
};

#[derive(Clone, SimpleObject)]
#[graphql(complex)]
pub struct Operation {
    pub id: ID,
    #[graphql(name = "type")]
    pub operation_type: String,
    pub detail: Option<Json<Value>>,
    pub undo_of_operation_id: Option<ID>,
    pub created_at: OffsetDateTime,
}

impl From<OperationDto> for Operation {
    fn from(dto: OperationDto) -> Self {
        Self {
            id: ID(dto.id),
            operation_type: dto.operation_type,
            detail: dto
                .detail
                .map(|detail| Json(serde_json::to_value(detail).expect("typed detail serializes"))),
            undo_of_operation_id: dto.undo_of_operation_id.map(ID),
            created_at: dto.created_at,
        }
    }
}

#[ComplexObject]
impl Operation {
    async fn book_changes(&self, ctx: &Context<'_>) -> Result<Vec<BookOperationChange>> {
        let loader = ctx.data_unchecked::<DataLoader<BookChangesByOperationLoader<HQ>>>();
        Ok(loader
            .load_one(self.id.to_string())
            .await?
            .unwrap_or_default())
    }

    async fn author_changes(&self, ctx: &Context<'_>) -> Result<Vec<AuthorOperationChange>> {
        let loader = ctx.data_unchecked::<DataLoader<AuthorChangesByOperationLoader<HQ>>>();
        Ok(loader
            .load_one(self.id.to_string())
            .await?
            .unwrap_or_default())
    }
}

#[derive(Clone, SimpleObject)]
pub struct BookRevision {
    pub book_id: ID,
    pub revision_number: i32,
    pub title: String,
    pub author_ids: Vec<ID>,
    pub isbn: String,
    pub read: bool,
    pub owned: bool,
    pub priority: i32,
    pub format: BookFormat,
    pub store: BookStore,
    pub purchase_date: Option<Date>,
    pub book_created_at: OffsetDateTime,
    pub book_updated_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

impl From<BookRevisionDto> for BookRevision {
    fn from(dto: BookRevisionDto) -> Self {
        Self {
            book_id: ID(dto.book_id),
            revision_number: dto.revision_number,
            title: dto.title,
            author_ids: dto.author_ids.into_iter().map(ID).collect(),
            isbn: dto.isbn,
            read: dto.read,
            owned: dto.owned,
            priority: dto.priority,
            format: dto.format.into(),
            store: dto.store.into(),
            purchase_date: dto.purchase_date,
            book_created_at: dto.book_created_at,
            book_updated_at: dto.book_updated_at,
            created_at: dto.created_at,
        }
    }
}

#[derive(Clone, SimpleObject)]
pub struct AuthorRevision {
    pub author_id: ID,
    pub revision_number: i32,
    pub name: String,
    pub yomi: String,
    pub author_created_at: OffsetDateTime,
    pub author_updated_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

impl From<AuthorRevisionDto> for AuthorRevision {
    fn from(dto: AuthorRevisionDto) -> Self {
        Self {
            author_id: ID(dto.author_id),
            revision_number: dto.revision_number,
            name: dto.name,
            yomi: dto.yomi,
            author_created_at: dto.author_created_at,
            author_updated_at: dto.author_updated_at,
            created_at: dto.created_at,
        }
    }
}

#[derive(Clone, SimpleObject)]
#[graphql(complex)]
pub struct BookOperationChange {
    #[graphql(skip)]
    pub operation_id: String,
    pub book_id: ID,
    #[graphql(skip)]
    pub before_revision_number: Option<i32>,
    #[graphql(skip)]
    pub after_revision_number: Option<i32>,
}

impl From<BookOperationChangeDto> for BookOperationChange {
    fn from(dto: BookOperationChangeDto) -> Self {
        Self {
            operation_id: dto.operation_id,
            book_id: ID(dto.book_id),
            before_revision_number: dto.before_revision_number,
            after_revision_number: dto.after_revision_number,
        }
    }
}

#[ComplexObject]
impl BookOperationChange {
    async fn before_revision(&self, ctx: &Context<'_>) -> Result<Option<BookRevision>> {
        revision_book(ctx, self, self.before_revision_number).await
    }
    async fn after_revision(&self, ctx: &Context<'_>) -> Result<Option<BookRevision>> {
        revision_book(ctx, self, self.after_revision_number).await
    }
}

async fn revision_book(
    ctx: &Context<'_>,
    change: &BookOperationChange,
    number: Option<i32>,
) -> Result<Option<BookRevision>> {
    let Some(number) = number else {
        return Ok(None);
    };
    let claims = ctx.data_unchecked::<Claims>();
    let history = ctx.data_unchecked::<HQ>();
    Ok(history
        .book_revision(&claims.sub, change.book_id.as_str(), number)
        .await?
        .map(Into::into))
}

#[derive(Clone, SimpleObject)]
#[graphql(complex)]
pub struct AuthorOperationChange {
    #[graphql(skip)]
    pub operation_id: String,
    pub author_id: ID,
    #[graphql(skip)]
    pub before_revision_number: Option<i32>,
    #[graphql(skip)]
    pub after_revision_number: Option<i32>,
}

impl From<AuthorOperationChangeDto> for AuthorOperationChange {
    fn from(dto: AuthorOperationChangeDto) -> Self {
        Self {
            operation_id: dto.operation_id,
            author_id: ID(dto.author_id),
            before_revision_number: dto.before_revision_number,
            after_revision_number: dto.after_revision_number,
        }
    }
}

#[ComplexObject]
impl AuthorOperationChange {
    async fn before_revision(&self, ctx: &Context<'_>) -> Result<Option<AuthorRevision>> {
        revision_author(ctx, self, self.before_revision_number).await
    }
    async fn after_revision(&self, ctx: &Context<'_>) -> Result<Option<AuthorRevision>> {
        revision_author(ctx, self, self.after_revision_number).await
    }
}

async fn revision_author(
    ctx: &Context<'_>,
    change: &AuthorOperationChange,
    number: Option<i32>,
) -> Result<Option<AuthorRevision>> {
    let Some(number) = number else {
        return Ok(None);
    };
    let claims = ctx.data_unchecked::<Claims>();
    let history = ctx.data_unchecked::<HQ>();
    Ok(history
        .author_revision(&claims.sub, change.author_id.as_str(), number)
        .await?
        .map(Into::into))
}

#[derive(SimpleObject)]
pub struct User {
    id: ID,
}

impl User {
    pub fn new(id: ID) -> Self {
        Self { id }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum BookFormat {
    EBook,
    Printed,
    Unknown,
}

impl From<CommonBookFormat> for BookFormat {
    fn from(book_format: CommonBookFormat) -> Self {
        match book_format {
            CommonBookFormat::EBook => BookFormat::EBook,
            CommonBookFormat::Printed => BookFormat::Printed,
            CommonBookFormat::Unknown => BookFormat::Unknown,
        }
    }
}

impl From<BookFormat> for CommonBookFormat {
    fn from(book_format: BookFormat) -> Self {
        match book_format {
            BookFormat::EBook => CommonBookFormat::EBook,
            BookFormat::Printed => CommonBookFormat::Printed,
            BookFormat::Unknown => CommonBookFormat::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum BookStore {
    Kindle,
    Unknown,
}

impl From<CommonBookStore> for BookStore {
    fn from(book_format: CommonBookStore) -> Self {
        match book_format {
            CommonBookStore::Kindle => BookStore::Kindle,
            CommonBookStore::Unknown => BookStore::Unknown,
        }
    }
}

impl From<BookStore> for CommonBookStore {
    fn from(book_format: BookStore) -> Self {
        match book_format {
            BookStore::Kindle => CommonBookStore::Kindle,
            BookStore::Unknown => CommonBookStore::Unknown,
        }
    }
}

#[derive(Clone, SimpleObject)]
#[graphql(complex)]
pub struct Book {
    pub id: String,
    pub title: String,
    #[graphql(skip)]
    pub author_ids: Vec<String>,
    pub isbn: String,
    pub read: bool,
    pub owned: bool,
    pub priority: i32,
    pub format: BookFormat,
    pub store: BookStore,
    pub purchase_date: Option<Date>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Book {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        title: String,
        author_ids: Vec<String>,
        isbn: String,
        read: bool,
        owned: bool,
        priority: i32,
        format: BookFormat,
        store: BookStore,
        purchase_date: Option<Date>,
        created_at: i64,
        updated_at: i64,
    ) -> Self {
        Self {
            id,
            title,
            author_ids,
            isbn,
            read,
            owned,
            priority,
            format,
            store,
            purchase_date,
            created_at,
            updated_at,
        }
    }
}

#[ComplexObject]
impl Book {
    async fn authors(&self, ctx: &Context<'_>) -> Result<Vec<Author>> {
        let loader = ctx.data_unchecked::<DataLoader<AuthorLoader<AQ>>>();
        let authors: Vec<Author> = loader
            .load_many(self.author_ids.clone()) // TODO cloneやめる
            .await?
            .into_values()
            .collect();

        Ok(authors)
    }
}

impl From<BookDto> for Book {
    fn from(book_dto: BookDto) -> Self {
        Self {
            id: book_dto.id,
            title: book_dto.title,
            author_ids: book_dto.author_ids,
            isbn: book_dto.isbn,
            read: book_dto.read,
            owned: book_dto.owned,
            priority: book_dto.priority,
            format: book_dto.format.into(),
            store: book_dto.store.into(),
            purchase_date: book_dto.purchase_date,
            created_at: book_dto.created_at.unix_timestamp(),
            updated_at: book_dto.updated_at.unix_timestamp(),
        }
    }
}

#[derive(InputObject)]
pub struct CreateBookInput {
    pub title: String,
    pub author_ids: Vec<String>,
    pub isbn: String,
    pub read: bool,
    pub owned: bool,
    pub priority: i32,
    pub format: BookFormat,
    pub store: BookStore,
    pub purchase_date: Option<Date>,
}

impl From<CreateBookInput> for CreateBookDto {
    fn from(book_input: CreateBookInput) -> Self {
        let CreateBookInput {
            title,
            author_ids,
            isbn,
            read,
            owned,
            priority,
            format,
            store,
            purchase_date,
        } = book_input;

        CreateBookDto {
            title,
            author_ids,
            isbn,
            read,
            owned,
            priority,
            format: format.into(),
            store: store.into(),
            purchase_date,
        }
    }
}

#[derive(InputObject)]
pub struct UpdateBookInput {
    pub id: String,
    pub title: String,
    pub author_ids: Vec<String>,
    pub isbn: String,
    pub read: bool,
    pub owned: bool,
    pub priority: i32,
    pub format: BookFormat,
    pub store: BookStore,
    pub purchase_date: Option<Date>,
}

impl From<UpdateBookInput> for UpdateBookDto {
    fn from(book_input: UpdateBookInput) -> Self {
        let UpdateBookInput {
            id,
            title,
            author_ids,
            isbn,
            read,
            owned,
            priority,
            format,
            store,
            purchase_date,
        } = book_input;

        UpdateBookDto {
            id,
            title,
            author_ids,
            isbn,
            read,
            owned,
            priority,
            format: format.into(),
            store: store.into(),
            purchase_date,
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex)]
pub struct Author {
    pub id: ID,
    pub name: String,
    pub yomi: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[ComplexObject]
impl Author {
    async fn books(&self, ctx: &Context<'_>) -> Result<Vec<Book>> {
        let loader = ctx.data_unchecked::<DataLoader<BooksByAuthorLoader<BQ>>>();
        Ok(loader
            .load_one(self.id.to_string())
            .await?
            .unwrap_or_default())
    }
}

impl Author {
    pub fn new(
        id: String,
        name: String,
        yomi: String,
        created_at: OffsetDateTime,
        updated_at: OffsetDateTime,
    ) -> Self {
        Self {
            id: ID(id),
            name,
            yomi,
            created_at,
            updated_at,
        }
    }
}

impl From<AuthorDto> for Author {
    fn from(author: AuthorDto) -> Self {
        let AuthorDto {
            id,
            name,
            yomi,
            created_at,
            updated_at,
        } = author;
        Author::new(id, name, yomi, created_at, updated_at)
    }
}

#[derive(InputObject)]
pub struct CreateAuthorInput {
    pub name: String,
    pub yomi: Option<String>,
}

impl CreateAuthorInput {
    pub fn new(name: String) -> Self {
        Self { name, yomi: None }
    }
}

impl From<CreateAuthorInput> for CreateAuthorDto {
    fn from(val: CreateAuthorInput) -> Self {
        CreateAuthorDto {
            name: val.name,
            yomi: val.yomi,
        }
    }
}

#[derive(InputObject)]
pub struct UpdateAuthorInput {
    pub id: ID,
    pub name: String,
    pub yomi: Option<String>,
}

impl From<UpdateAuthorInput> for UpdateAuthorDto {
    fn from(val: UpdateAuthorInput) -> Self {
        UpdateAuthorDto {
            id: val.id.to_string(),
            name: val.name,
            yomi: val.yomi,
        }
    }
}

#[derive(InputObject)]
pub struct ImportBookInput {
    /// Title of the book.
    pub title: String,
    /// Names of the authors. Authors will be created if they do not exist.
    pub author_names: Vec<String>,
    /// ISBN of the book.
    pub isbn: String,
    /// Whether the book has been read.
    pub read: bool,
    /// Whether the book is owned.
    pub owned: bool,
    /// Priority value ranging from 0 to 100.
    pub priority: i32,
    /// Format of the book.
    pub format: BookFormat,
    /// Store where the book was purchased or obtained.
    pub store: BookStore,
    /// Calendar date on which the book was purchased.
    pub purchase_date: Option<Date>,
}

impl From<ImportBookInput> for ImportBookEntryDto {
    fn from(input: ImportBookInput) -> Self {
        ImportBookEntryDto {
            title: input.title,
            author_names: input.author_names,
            isbn: input.isbn,
            read: input.read,
            owned: input.owned,
            priority: input.priority,
            format: input.format.into(),
            store: input.store.into(),
            purchase_date: input.purchase_date,
        }
    }
}

#[derive(SimpleObject)]
pub struct BookMutationPayload {
    pub book: Book,
    pub operation_id: ID,
    pub revision_number: i32,
}

impl BookMutationPayload {
    pub fn new(book: Book, operation_id: ID, revision_number: i32) -> Self {
        Self {
            book,
            operation_id,
            revision_number,
        }
    }
}

#[derive(SimpleObject)]
pub struct AuthorMutationPayload {
    pub author: Author,
    pub operation_id: ID,
    pub revision_number: i32,
}

#[derive(SimpleObject)]
pub struct MergeAuthorPayload {
    pub author: Author,
    pub operation_id: ID,
}

impl AuthorMutationPayload {
    pub fn new(author: Author, operation_id: ID, revision_number: i32) -> Self {
        Self {
            author,
            operation_id,
            revision_number,
        }
    }
}

#[derive(SimpleObject)]
pub struct DeleteBookPayload {
    pub book_id: ID,
    pub operation_id: ID,
}

#[derive(SimpleObject)]
pub struct DeleteAuthorPayload {
    pub author_id: ID,
    pub operation_id: ID,
}

#[derive(SimpleObject)]
pub struct ImportBooksPayload {
    pub books: Vec<Book>,
    pub operation_id: ID,
}

#[derive(Clone, SimpleObject)]
pub struct UndoOperationPayload {
    pub operation_id: ID,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum ImportAuthorStatus {
    Existing,
    New,
}

impl From<ImportAuthorStatusDto> for ImportAuthorStatus {
    fn from(status: ImportAuthorStatusDto) -> Self {
        match status {
            ImportAuthorStatusDto::Existing => Self::Existing,
            ImportAuthorStatusDto::New => Self::New,
        }
    }
}

#[derive(SimpleObject)]
pub struct ImportAuthorPreview {
    pub name: String,
    pub status: ImportAuthorStatus,
}

impl From<ImportAuthorPreviewDto> for ImportAuthorPreview {
    fn from(dto: ImportAuthorPreviewDto) -> Self {
        Self {
            name: dto.name,
            status: dto.status.into(),
        }
    }
}

#[derive(SimpleObject)]
pub struct ImportBookPreview {
    pub title: String,
    pub authors: Vec<ImportAuthorPreview>,
    pub isbn: String,
    pub read: bool,
    pub owned: bool,
    pub priority: i32,
    pub format: BookFormat,
    pub store: BookStore,
    pub purchase_date: Option<Date>,
}

impl From<ImportBookPreviewDto> for ImportBookPreview {
    fn from(dto: ImportBookPreviewDto) -> Self {
        Self {
            title: dto.title,
            authors: dto
                .authors
                .into_iter()
                .map(ImportAuthorPreview::from)
                .collect(),
            isbn: dto.isbn,
            read: dto.read,
            owned: dto.owned,
            priority: dto.priority,
            format: dto.format.into(),
            store: dto.store.into(),
            purchase_date: dto.purchase_date,
        }
    }
}

#[derive(SimpleObject)]
pub struct ImportBooksPreview {
    pub books: Vec<ImportBookPreview>,
}

impl From<ImportBooksPreviewDto> for ImportBooksPreview {
    fn from(dto: ImportBooksPreviewDto) -> Self {
        Self {
            books: dto.books.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(SimpleObject)]
pub struct RestoreBookPayload {
    pub book: Option<Book>,
    pub operation_id: ID,
    pub revision_number: i32,
}

#[derive(SimpleObject)]
pub struct RestoreAuthorPayload {
    pub author: Option<Author>,
    pub operation_id: ID,
    pub revision_number: i32,
}
