use std::sync::Arc;

use async_graphql::{Context, ID, Object};

use crate::{
    presentation::{error::PresentationalError, extractor::claims::Claims},
    use_case::traits::{
        author::AuthorCommandUseCase, book::BookCommandUseCase, user::UserCommandUseCase,
    },
};

use super::object::{
    Author, AuthorMutationPayload, Book, BookMutationPayload, CreateAuthorInput, CreateBookInput,
    DeleteAuthorPayload, DeleteBookPayload, ImportBookInput, ImportBooksPayload,
    ImportBooksPreview, MergeAuthorPayload, RestoreAuthorPayload, RestoreBookPayload,
    UpdateAuthorInput, UpdateBookInput, User,
};

pub struct Mutation<UC, BC, AC> {
    user_command: UC,
    book_command: BC,
    author_command: AC,
}

impl<UC, BC, AC> Mutation<UC, BC, AC> {
    pub fn new(user_command: UC, book_command: BC, author_command: AC) -> Self {
        Self {
            user_command,
            book_command,
            author_command,
        }
    }
}

#[Object]
impl<UC, BC, AC> Mutation<UC, BC, AC>
where
    UC: UserCommandUseCase,
    BC: BookCommandUseCase,
    AC: AuthorCommandUseCase,
{
    async fn register_user(&self, ctx: &Context<'_>) -> Result<User, PresentationalError> {
        let claims = get_claims(ctx)?;
        let user = self.user_command.register(&claims.sub).await?;
        Ok(User::new(ID(user.id)))
    }

    async fn create_book(
        &self,
        ctx: &Context<'_>,
        book_data: CreateBookInput,
    ) -> Result<BookMutationPayload, PresentationalError> {
        let claims = get_claims(ctx)?;
        let book = self
            .book_command
            .create(&claims.sub, book_data.into())
            .await?;

        Ok(BookMutationPayload::new(
            book.value.into(),
            ID(book.event_set_id),
            ID(book.event_id.to_string()),
        ))
    }

    async fn update_book(
        &self,
        ctx: &Context<'_>,
        book_data: UpdateBookInput,
    ) -> Result<BookMutationPayload, PresentationalError> {
        let claims = get_claims(ctx)?;
        let book = self
            .book_command
            .update(&claims.sub, book_data.into())
            .await?;

        Ok(BookMutationPayload::new(
            book.value.into(),
            ID(book.event_set_id),
            ID(book.event_id.to_string()),
        ))
    }

    async fn delete_book(
        &self,
        ctx: &Context<'_>,
        book_id: ID,
    ) -> Result<DeleteBookPayload, PresentationalError> {
        let claims = get_claims(ctx)?;
        let result = self
            .book_command
            .delete(&claims.sub, book_id.as_str())
            .await?;

        Ok(DeleteBookPayload {
            book_id: ID(result.value),
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
            .author_command
            .create(&claims.sub, author_data.into())
            .await?;
        Ok(AuthorMutationPayload::new(
            author.value.into(),
            ID(author.event_set_id),
            ID(author.event_id.to_string()),
        ))
    }

    async fn update_author(
        &self,
        ctx: &Context<'_>,
        author_data: UpdateAuthorInput,
    ) -> Result<AuthorMutationPayload, PresentationalError> {
        let claims = get_claims(ctx)?;
        let author = self
            .author_command
            .update(&claims.sub, author_data.into())
            .await?;
        Ok(AuthorMutationPayload::new(
            author.value.into(),
            ID(author.event_set_id),
            ID(author.event_id.to_string()),
        ))
    }

    async fn delete_author(
        &self,
        ctx: &Context<'_>,
        author_id: ID,
    ) -> Result<DeleteAuthorPayload, PresentationalError> {
        let claims = get_claims(ctx)?;
        let result = self
            .author_command
            .delete(&claims.sub, author_id.as_str())
            .await?;
        Ok(DeleteAuthorPayload {
            author_id: ID(result.value),
            event_set_id: ID(result.event_set_id),
        })
    }

    async fn merge_author(
        &self,
        ctx: &Context<'_>,
        source_author_id: ID,
        destination_author_id: ID,
    ) -> Result<MergeAuthorPayload, PresentationalError> {
        let claims = get_claims(ctx)?;
        let result = self
            .author_command
            .merge(
                &claims.sub,
                crate::use_case::dto::author::MergeAuthorInputDto {
                    source_author_id: source_author_id.to_string(),
                    destination_author_id: destination_author_id.to_string(),
                },
            )
            .await?;
        Ok(MergeAuthorPayload {
            author: result.value.into(),
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
        let book = self.book_command.restore(&claims.sub, eid).await?;
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
        let author = self.author_command.restore(&claims.sub, eid).await?;
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
            .book_command
            .import(&claims.sub, books.into_iter().map(Into::into).collect())
            .await?;
        Ok(ImportBooksPayload {
            books: books.value.into_iter().map(Book::from).collect(),
            event_set_id: ID(books.event_set_id),
        })
    }

    /// Executes the book import path and rolls the complete transaction back.
    async fn preview_book_import(
        &self,
        ctx: &Context<'_>,
        books: Vec<ImportBookInput>,
    ) -> Result<ImportBooksPreview, PresentationalError> {
        let claims = get_claims(ctx)?;
        Ok(self
            .book_command
            .preview_import(&claims.sub, books.into_iter().map(Into::into).collect())
            .await?
            .into())
    }
}

fn get_claims<'a>(ctx: &Context<'a>) -> Result<&'a Claims, PresentationalError> {
    ctx.data::<Claims>()
        .map_err(|err| PresentationalError::OtherError(Arc::new(anyhow::anyhow!(err.message))))
}
