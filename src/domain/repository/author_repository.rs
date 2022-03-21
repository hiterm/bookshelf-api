use async_trait::async_trait;

use crate::domain::{
    entity::{
        author::{Author, AuthorId},
        user::UserId,
    },
    error::DomainError,
};

#[async_trait]
pub trait AuthorRepository: Send + Sync {
    async fn create(&self, user_id: &UserId, author: &Author) -> Result<(), DomainError>;
    async fn find_by_id(
        &self,
        user_id: &UserId,
        author_id: &AuthorId,
    ) -> Result<Option<Author>, DomainError>;
}

#[cfg(test)]
pub mod tests {
    use async_trait::async_trait;
    use mockall::mock;

    use crate::domain::{
        self,
        entity::{author::AuthorId, user::UserId},
        error::DomainError,
        repository::author_repository::AuthorRepository,
    };

    mock! {
        pub AuthorRepository {}
        #[async_trait]
        impl AuthorRepository for AuthorRepository {
            async fn create(&self, user_id: &UserId, author: &domain::entity::author::Author) -> Result<(), DomainError>;
            async fn find_by_id(
                &self,
                user_id: &UserId,
                author_id: &AuthorId,
            ) -> Result<Option<domain::entity::author::Author>, DomainError>;
        }
    }
}
