use async_graphql::{Context, Object, ID};

use crate::{
    extractors::Claims, presentational::error::error::PresentationalError,
    use_case::use_case::query::QueryUseCase,
};

use super::object::{Author, User};

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
    async fn login_user(&self, ctx: &Context<'_>) -> Result<User, PresentationalError> {
        let claims = ctx
            .data::<Claims>()
            .map_err(|err| PresentationalError::OtherError(anyhow::anyhow!(err.message)))?;
        let user = self.query_use_case.find_user_by_id(&claims.sub).await?;
        Ok(User::new(ID(user.id)))
    }

    async fn author(&self, ctx: &Context<'_>, id: ID) -> Result<Author, PresentationalError> {
        let claims = ctx
            .data::<Claims>()
            .map_err(|err| PresentationalError::OtherError(anyhow::anyhow!(err.message)))?;
        let author = self
            .query_use_case
            .find_author_by_id(&claims.sub, id.as_str())
            .await?;
        Ok(Author::new(author.id, author.name))
    }
}
