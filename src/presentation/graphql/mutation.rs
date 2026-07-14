use std::sync::Arc;

use async_graphql::{Context, ID, Object};

use crate::{
    presentation::{error::PresentationalError, extractor::claims::Claims},
    use_case::traits::mutation::MutationUseCase,
};

use super::object::{
    Author, AuthorMutationPayload, Book, BookMutationPayload, CreateAuthorInput, CreateBookInput,
    DeleteAuthorPayload, DeleteBookPayload, ImportBookInput, ImportBooksPayload,
    RestoreAuthorPayload, RestoreBookPayload, UpdateAuthorInput, UpdateBookInput, User,
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
    ) -> Result<BookMutationPayload, PresentationalError> {
        let claims = get_claims(ctx)?;
        let book = self
            .mutation_use_case
            .create_book(&claims.sub, book_data.into())
            .await?;

        Ok(BookMutationPayload::new(
            book.value.into(),
            ID(book.event_set_id),
        ))
    }

    async fn update_book(
        &self,
        ctx: &Context<'_>,
        book_data: UpdateBookInput,
    ) -> Result<BookMutationPayload, PresentationalError> {
        let claims = get_claims(ctx)?;
        let book = self
            .mutation_use_case
            .update_book(&claims.sub, book_data.into())
            .await?;

        Ok(BookMutationPayload::new(
            book.value.into(),
            ID(book.event_set_id),
        ))
    }

    async fn delete_book(
        &self,
        ctx: &Context<'_>,
        book_id: ID,
    ) -> Result<DeleteBookPayload, PresentationalError> {
        let claims = get_claims(ctx)?;
        let result = self
            .mutation_use_case
            .delete_book(&claims.sub, book_id.as_str())
            .await?;

        let book_id = ID(result.value);
        Ok(DeleteBookPayload {
            book_id: book_id.clone(),
            id: book_id,
            event_set_id: ID(result.event_set_id),
        })
    }

    async fn create_author(
        &self,
        ctx: &Context<'_>,
        author_data: CreateAuthorInput,
    ) -> Result<AuthorMutationPayload, PresentationalError> {
        let claims = get_claims(ctx)?;
        let author = self
            .mutation_use_case
            .create_author(&claims.sub, author_data.into())
            .await?;
        Ok(AuthorMutationPayload::new(
            author.value.into(),
            ID(author.event_set_id),
        ))
    }

    async fn update_author(
        &self,
        ctx: &Context<'_>,
        author_data: UpdateAuthorInput,
    ) -> Result<AuthorMutationPayload, PresentationalError> {
        let claims = get_claims(ctx)?;
        let author = self
            .mutation_use_case
            .update_author(&claims.sub, author_data.into())
            .await?;
        Ok(AuthorMutationPayload::new(
            author.value.into(),
            ID(author.event_set_id),
        ))
    }

    async fn delete_author(
        &self,
        ctx: &Context<'_>,
        author_id: ID,
    ) -> Result<DeleteAuthorPayload, PresentationalError> {
        let claims = get_claims(ctx)?;
        let result = self
            .mutation_use_case
            .delete_author(&claims.sub, author_id.as_str())
            .await?;
        let author_id = ID(result.value);
        Ok(DeleteAuthorPayload {
            author_id: author_id.clone(),
            id: author_id,
            event_set_id: ID(result.event_set_id),
        })
    }

    async fn restore_book(
        &self,
        ctx: &Context<'_>,
        event_id: ID,
    ) -> Result<RestoreBookPayload, PresentationalError> {
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
        Ok(RestoreBookPayload {
            book: book.value.map(Book::from),
            event_set_id: ID(book.event_set_id),
        })
    }

    async fn restore_author(
        &self,
        ctx: &Context<'_>,
        event_id: ID,
    ) -> Result<RestoreAuthorPayload, PresentationalError> {
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
        Ok(RestoreAuthorPayload {
            author: author.value.map(Author::from),
            event_set_id: ID(author.event_set_id),
        })
    }

    /// Imports multiple books. Creates authors if they do not exist.
    async fn import_books(
        &self,
        ctx: &Context<'_>,
        books: Vec<ImportBookInput>,
    ) -> Result<ImportBooksPayload, PresentationalError> {
        let claims = get_claims(ctx)?;
        let books = self
            .mutation_use_case
            .import_books(&claims.sub, books.into_iter().map(Into::into).collect())
            .await?;
        Ok(ImportBooksPayload {
            books: books.value.into_iter().map(Book::from).collect(),
            event_set_id: ID(books.event_set_id),
        })
    }
}

fn get_claims<'a>(ctx: &Context<'a>) -> Result<&'a Claims, PresentationalError> {
    ctx.data::<Claims>()
        .map_err(|err| PresentationalError::OtherError(Arc::new(anyhow::anyhow!(err.message))))
}
