use std::sync::Arc;

use async_graphql::{Context, ID, Object};

use crate::{
    presentation::{error::PresentationalError, extractor::claims::Claims},
    use_case::traits::{
        author::AuthorQueryUseCase, book::BookQueryUseCase, history::HistoryQueryUseCase,
        user::UserQueryUseCase,
    },
};

use super::object::{Author, AuthorRevision, Book, BookRevision, Operation, User};

pub struct Query<UQ, BQ, AQ, HQ> {
    user_query: UQ,
    book_query: BQ,
    author_query: AQ,
    history_query: HQ,
}

impl<UQ, BQ, AQ, HQ> Query<UQ, BQ, AQ, HQ> {
    pub fn new(user_query: UQ, book_query: BQ, author_query: AQ, history_query: HQ) -> Self {
        Self {
            user_query,
            book_query,
            author_query,
            history_query,
        }
    }
}

#[Object]
impl<UQ, BQ, AQ, HQ> Query<UQ, BQ, AQ, HQ>
where
    UQ: UserQueryUseCase,
    BQ: BookQueryUseCase,
    AQ: AuthorQueryUseCase,
    HQ: HistoryQueryUseCase,
{
    async fn operations(&self, ctx: &Context<'_>) -> Result<Vec<Operation>, PresentationalError> {
        let claims = get_claims(ctx)?;
        Ok(self
            .history_query
            .operations(&claims.sub)
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    async fn operation(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> Result<Option<Operation>, PresentationalError> {
        let claims = get_claims(ctx)?;
        Ok(self
            .history_query
            .operation(&claims.sub, id.as_str())
            .await?
            .map(Into::into))
    }

    async fn book_revisions(
        &self,
        ctx: &Context<'_>,
        book_id: ID,
    ) -> Result<Vec<BookRevision>, PresentationalError> {
        let claims = get_claims(ctx)?;
        Ok(self
            .history_query
            .book_revisions(&claims.sub, book_id.as_str())
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    async fn book_revision(
        &self,
        ctx: &Context<'_>,
        book_id: ID,
        revision_number: i32,
    ) -> Result<Option<BookRevision>, PresentationalError> {
        let claims = get_claims(ctx)?;
        Ok(self
            .history_query
            .book_revision(&claims.sub, book_id.as_str(), revision_number)
            .await?
            .map(Into::into))
    }

    async fn author_revisions(
        &self,
        ctx: &Context<'_>,
        author_id: ID,
    ) -> Result<Vec<AuthorRevision>, PresentationalError> {
        let claims = get_claims(ctx)?;
        Ok(self
            .history_query
            .author_revisions(&claims.sub, author_id.as_str())
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    async fn author_revision(
        &self,
        ctx: &Context<'_>,
        author_id: ID,
        revision_number: i32,
    ) -> Result<Option<AuthorRevision>, PresentationalError> {
        let claims = get_claims(ctx)?;
        Ok(self
            .history_query
            .author_revision(&claims.sub, author_id.as_str(), revision_number)
            .await?
            .map(Into::into))
    }
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
}

fn get_claims<'a>(ctx: &Context<'a>) -> Result<&'a Claims, PresentationalError> {
    ctx.data::<Claims>()
        .map_err(|err| PresentationalError::OtherError(Arc::new(anyhow::anyhow!(err.message))))
}
