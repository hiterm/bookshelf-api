use std::sync::Arc;

use async_graphql::{Context, ID, Object};

use crate::{
    presentation::{error::PresentationalError, extractor::claims::Claims},
    use_case::traits::mutation::MutationUseCase,
};

use super::object::{
    Author, Book, CreateAuthorInput, CreateBookInput, ImportBookInput, UpdateAuthorInput,
    UpdateBookInput, User,
};

pub struct Mutation<MUC> {
    mutation_use_case: MUC,
}

impl<MUC> Mutation<MUC> {
    pub fn new(mutation_use_case: MUC) -> Self {
        Self { mutation_use_case }
    }
}

#[Object]
impl<MUC> Mutation<MUC>
where
    MUC: MutationUseCase,
{
    async fn register_user(&self, ctx: &Context<'_>) -> Result<User, PresentationalError> {
        let claims = get_claims(ctx)?;
        let user = self.mutation_use_case.register_user(&claims.sub).await?;
        Ok(User::new(ID(user.id)))
    }

    async fn create_book(
        &self,
        ctx: &Context<'_>,
        book_data: CreateBookInput,
    ) -> Result<Book, PresentationalError> {
        let claims = get_claims(ctx)?;
        let book = self
            .mutation_use_case
            .create_book(&claims.sub, book_data.into())
            .await?;

        Ok(book.into())
    }

    async fn update_book(
        &self,
        ctx: &Context<'_>,
        book_data: UpdateBookInput,
    ) -> Result<Book, PresentationalError> {
        let claims = get_claims(ctx)?;
        let book = self
            .mutation_use_case
            .update_book(&claims.sub, book_data.into())
            .await?;

        Ok(book.into())
    }

    async fn delete_book(
        &self,
        ctx: &Context<'_>,
        book_id: ID,
    ) -> Result<String, PresentationalError> {
        let claims = get_claims(ctx)?;
        self.mutation_use_case
            .delete_book(&claims.sub, book_id.as_str())
            .await?;

        Ok(book_id.to_string())
    }

    async fn create_author(
        &self,
        ctx: &Context<'_>,
        author_data: CreateAuthorInput,
    ) -> Result<Author, PresentationalError> {
        let claims = get_claims(ctx)?;
        let author = self
            .mutation_use_case
            .create_author(&claims.sub, author_data.into())
            .await?;
        Ok(author.into())
    }

    async fn update_author(
        &self,
        ctx: &Context<'_>,
        author_data: UpdateAuthorInput,
    ) -> Result<Author, PresentationalError> {
        let claims = get_claims(ctx)?;
        let author = self
            .mutation_use_case
            .update_author(&claims.sub, author_data.into())
            .await?;
        Ok(author.into())
    }

    async fn delete_author(
        &self,
        ctx: &Context<'_>,
        author_id: ID,
    ) -> Result<String, PresentationalError> {
        let claims = get_claims(ctx)?;
        self.mutation_use_case
            .delete_author(&claims.sub, author_id.as_str())
            .await?;
        Ok(author_id.to_string())
    }

    async fn restore_book(
        &self,
        ctx: &Context<'_>,
        event_id: ID,
    ) -> Result<Option<Book>, PresentationalError> {
        let claims = get_claims(ctx)?;
        let eid: i64 = event_id.parse().map_err(|_| {
            PresentationalError::OtherError(std::sync::Arc::new(anyhow::anyhow!(
                "event_id must be an integer"
            )))
        })?;
        let book = self
            .mutation_use_case
            .restore_book(&claims.sub, eid)
            .await?;
        Ok(book.map(Book::from))
    }

    async fn restore_author(
        &self,
        ctx: &Context<'_>,
        event_id: ID,
    ) -> Result<Option<Author>, PresentationalError> {
        let claims = get_claims(ctx)?;
        let eid: i64 = event_id.parse().map_err(|_| {
            PresentationalError::OtherError(std::sync::Arc::new(anyhow::anyhow!(
                "event_id must be an integer"
            )))
        })?;
        let author = self
            .mutation_use_case
            .restore_author(&claims.sub, eid)
            .await?;
        Ok(author.map(Author::from))
    }

    async fn import_books(
        &self,
        ctx: &Context<'_>,
        books: Vec<ImportBookInput>,
    ) -> Result<Vec<Book>, PresentationalError> {
        let claims = get_claims(ctx)?;
        let books = self
            .mutation_use_case
            .import_books(&claims.sub, books.into_iter().map(Into::into).collect())
            .await?;
        Ok(books.into_iter().map(Book::from).collect())
    }
}

fn get_claims<'a>(ctx: &Context<'a>) -> Result<&'a Claims, PresentationalError> {
    ctx.data::<Claims>()
        .map_err(|err| PresentationalError::OtherError(Arc::new(anyhow::anyhow!(err.message))))
}
