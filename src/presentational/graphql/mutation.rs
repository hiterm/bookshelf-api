use async_graphql::{Context, Object, ID};

use crate::{
    extractors::Claims, presentational::error::error::PresentationalError,
    use_case::use_case::mutation::MutationUseCase,
};

use super::object::User;

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
        let claims = ctx
            .data::<Claims>()
            .map_err(|err| PresentationalError::OtherError(anyhow::anyhow!(err.message)))?;
        let user = self.mutation_use_case.register_user(&claims.sub).await?;
        Ok(User::new(ID(user.id)))
    }
}
