use std::sync::Arc;

use async_graphql::{Context, Object, ID};

use crate::{
    extractors::Claims, presentational::error::PresentationalError,
    use_case::traits::mutation::MutationUseCase,
};

use super::object::{Author, Book, CreateAuthorInput, CreateBookInput, UpdateBookInput, User};

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
        book_data: UpdateBookInput,
    ) -> Result<String, PresentationalError> {
        let claims = get_claims(ctx)?;
        self.mutation_use_case
            .delete_book(&claims.sub, &book_data.id)
            .await?;

        Ok(book_data.id)
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
}

fn get_claims<'a>(ctx: &Context<'a>) -> Result<&'a Claims, PresentationalError> {
    ctx.data::<Claims>()
        .map_err(|err| PresentationalError::OtherError(Arc::new(anyhow::anyhow!(err.message))))
}
