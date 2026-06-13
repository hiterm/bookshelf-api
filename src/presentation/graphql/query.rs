use std::sync::Arc;

use async_graphql::{Context, ID, Object};

use crate::{
    presentation::{error::PresentationalError, extractor::claims::Claims},
    use_case::traits::query::QueryUseCase,
};

use super::object::{
    Author, AuthorEventEntry, Book, BookEventEntry, EventSetDetail, EventSetEntry, User,
};

pub struct Query<QUC> {
    query_use_case: QUC,
}

impl<QUC> Query<QUC> {
    pub fn new(query_use_case: QUC) -> Self {
        Query { query_use_case }
    }
}

#[Object]
impl<QUC> Query<QUC>
where
    QUC: QueryUseCase,
{
    async fn logged_in_user(&self, ctx: &Context<'_>) -> Result<Option<User>, PresentationalError> {
        let claims = get_claims(ctx)?;
        let user = self.query_use_case.find_user_by_id(&claims.sub).await?;
        Ok(user.map(|user| User::new(ID(user.id))))
    }

    async fn book(&self, ctx: &Context<'_>, id: ID) -> Result<Option<Book>, PresentationalError> {
        let claims = get_claims(ctx)?;
        let book = self
            .query_use_case
            .find_book_by_id(&claims.sub, id.as_str())
            .await?;

        Ok(book.map(Book::from))
    }

    async fn books(&self, ctx: &Context<'_>) -> Result<Vec<Book>, PresentationalError> {
        let claims = get_claims(ctx)?;
        let books = self.query_use_case.find_all_books(&claims.sub).await?;
        let books: Vec<Book> = books.into_iter().map(Book::from).collect();

        Ok(books)
    }

    async fn author(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> Result<Option<Author>, PresentationalError> {
        let claims = get_claims(ctx)?;
        let author = self
            .query_use_case
            .find_author_by_id(&claims.sub, id.as_str())
            .await?;
        Ok(author.map(|author| Author::new(author.id, author.name)))
    }

    async fn authors(&self, ctx: &Context<'_>) -> Result<Vec<Author>, PresentationalError> {
        let claims = get_claims(ctx)?;
        let authors = self.query_use_case.find_all_authors(&claims.sub).await?;
        let authors: Vec<Author> = authors.into_iter().map(Author::from).collect();
        Ok(authors)
    }

    /// Returns the change history for a book.
    /// Entries are sorted by `changedAt` in descending order (newest first).
    async fn book_events(
        &self,
        ctx: &Context<'_>,
        book_id: ID,
    ) -> Result<Vec<BookEventEntry>, PresentationalError> {
        let claims = get_claims(ctx)?;
        let entries = self
            .query_use_case
            .list_book_events(&claims.sub, book_id.as_str())
            .await?;
        Ok(entries.into_iter().map(BookEventEntry::from).collect())
    }

    /// Returns the change history for an author.
    /// Entries are sorted by `changedAt` in descending order (newest first).
    async fn author_events(
        &self,
        ctx: &Context<'_>,
        author_id: ID,
    ) -> Result<Vec<AuthorEventEntry>, PresentationalError> {
        let claims = get_claims(ctx)?;
        let entries = self
            .query_use_case
            .list_author_events(&claims.sub, author_id.as_str())
            .await?;
        Ok(entries.into_iter().map(AuthorEventEntry::from).collect())
    }

    /// Returns the logged-in user's event sets, newest first.
    async fn event_sets(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<EventSetEntry>, PresentationalError> {
        let claims = get_claims(ctx)?;
        let sets = self.query_use_case.list_event_sets(&claims.sub).await?;
        Ok(sets.into_iter().map(EventSetEntry::from).collect())
    }

    /// Returns a single event set with nested events, or null if not found.
    async fn event_set(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> Result<Option<EventSetDetail>, PresentationalError> {
        let claims = get_claims(ctx)?;
        let detail = self
            .query_use_case
            .find_event_set(&claims.sub, id.as_str())
            .await?;
        Ok(detail.map(EventSetDetail::from))
    }
}

fn get_claims<'a>(ctx: &Context<'a>) -> Result<&'a Claims, PresentationalError> {
    ctx.data::<Claims>()
        .map_err(|err| PresentationalError::OtherError(Arc::new(anyhow::anyhow!(err.message))))
}
