use std::collections::HashMap;

use async_trait::async_trait;
use mockall::automock;
use sqlx::PgConnection;

use crate::domain::{
    entity::{
        author::{Author, AuthorId},
        user::UserId,
    },
    error::DomainError,
};

#[automock]
#[async_trait]
pub trait AuthorRepository: Send + Sync + 'static {
    async fn create(
        &self,
        conn: &mut PgConnection,
        user_id: &UserId,
        author: &Author,
    ) -> Result<(), DomainError>;
    async fn find_by_id(
        &self,
        conn: &mut PgConnection,
        user_id: &UserId,
        author_id: &AuthorId,
    ) -> Result<Option<Author>, DomainError>;
    async fn find_all(
        &self,
        conn: &mut PgConnection,
        user_id: &UserId,
    ) -> Result<Vec<Author>, DomainError>;
    async fn find_by_ids_as_hash_map(
        &self,
        conn: &mut PgConnection,
        user_id: &UserId,
        author_ids: &[AuthorId],
    ) -> Result<HashMap<AuthorId, Author>, DomainError>;
    async fn update(
        &self,
        conn: &mut PgConnection,
        user_id: &UserId,
        author: &Author,
    ) -> Result<(), DomainError>;
    async fn delete(
        &self,
        conn: &mut PgConnection,
        user_id: &UserId,
        author_id: &AuthorId,
    ) -> Result<(), DomainError>;
    // Upserts or deletes the entity and records a restore event in one transaction.
    // author=Some means upsert; author=None means delete (only author_id is used).
    async fn restore(
        &self,
        conn: &mut PgConnection,
        user_id: &UserId,
        source_event_id: i64,
        author: Option<Author>,
    ) -> Result<(), DomainError>;
}
