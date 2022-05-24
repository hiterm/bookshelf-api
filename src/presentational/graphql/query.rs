use std::sync::Arc;

use async_graphql::{Context, Object, ID};

use crate::{
    extractors::Claims, presentational::error::PresentationalError,
    use_case::use_case::query::QueryUseCase,
};

use super::object::{Author, Book, User};

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
}

fn get_claims<'a>(ctx: &Context<'a>) -> Result<&'a Claims, PresentationalError> {
    ctx.data::<Claims>()
        .map_err(|err| PresentationalError::OtherError(Arc::new(anyhow::anyhow!(err.message))))
}
