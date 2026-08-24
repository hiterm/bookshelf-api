use std::sync::Arc;

use async_graphql::{Context, ID, Object};

use crate::{
    presentation::{error::PresentationalError, extractor::claims::Claims},
    use_case::traits::{
        author::AuthorQueryUseCase, book::BookQueryUseCase, event::EventQueryUseCase,
        user::UserQueryUseCase,
    },
};

use super::object::{Author, AuthorEventEntry, Book, BookEventEntry, EventSet, User};

pub struct Query<UQ, BQ, AQ, EQ> {
    user_query: UQ,
    book_query: BQ,
    author_query: AQ,
    event_query: EQ,
}

impl<UQ, BQ, AQ, EQ> Query<UQ, BQ, AQ, EQ> {
    pub fn new(user_query: UQ, book_query: BQ, author_query: AQ, event_query: EQ) -> Self {
        Self {
            user_query,
            book_query,
            author_query,
            event_query,
        }
    }
}

#[Object]
impl<UQ, BQ, AQ, EQ> Query<UQ, BQ, AQ, EQ>
where
    UQ: UserQueryUseCase,
    BQ: BookQueryUseCase,
    AQ: AuthorQueryUseCase,
    EQ: EventQueryUseCase,
{
    async fn logged_in_user(&self, ctx: &Context<'_>) -> Result<Option<User>, PresentationalError> {
        let claims = get_claims(ctx)?;
        let user = self.user_query.find_by_id(&claims.sub).await?;
        Ok(user.map(|user| User::new(ID(user.id))))
    }

    async fn book(&self, ctx: &Context<'_>, id: ID) -> Result<Option<Book>, PresentationalError> {
        let claims = get_claims(ctx)?;
        let book = self.book_query.find_by_id(&claims.sub, id.as_str()).await?;

        Ok(book.map(Book::from))
    }

    async fn books(&self, ctx: &Context<'_>) -> Result<Vec<Book>, PresentationalError> {
        let claims = get_claims(ctx)?;
        let books = self.book_query.find_all(&claims.sub).await?;
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
            .author_query
            .find_by_id(&claims.sub, id.as_str())
            .await?;
        Ok(author.map(Author::from))
    }

    async fn authors(&self, ctx: &Context<'_>) -> Result<Vec<Author>, PresentationalError> {
        let claims = get_claims(ctx)?;
        let authors = self.author_query.find_all(&claims.sub).await?;
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
            .event_query
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
            .event_query
            .list_author_events(&claims.sub, author_id.as_str())
            .await?;
        Ok(entries.into_iter().map(AuthorEventEntry::from).collect())
    }

    /// Returns the logged-in user's event sets, newest first.
    async fn event_sets(&self, ctx: &Context<'_>) -> Result<Vec<EventSet>, PresentationalError> {
        let claims = get_claims(ctx)?;
        let sets = self.event_query.list_event_sets(&claims.sub).await?;
        Ok(sets.into_iter().map(EventSet::from).collect())
    }

    /// Returns a single event set with nested events, or null if not found.
    async fn event_set(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> Result<Option<EventSet>, PresentationalError> {
        let claims = get_claims(ctx)?;
        let event_set = self
            .event_query
            .find_event_set(&claims.sub, id.as_str())
            .await?;
        Ok(event_set.map(EventSet::from))
    }
}

fn get_claims<'a>(ctx: &Context<'a>) -> Result<&'a Claims, PresentationalError> {
    ctx.data::<Claims>()
        .map_err(|err| PresentationalError::OtherError(Arc::new(anyhow::anyhow!(err.message))))
}
