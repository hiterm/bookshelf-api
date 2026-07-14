use async_trait::async_trait;
use mockall::automock;

use uuid::Uuid;

use crate::domain::{
    entity::{event::EventSetOperation, user::UserId},
    error::DomainError,
};

pub trait TransactionEventSet {
    fn event_set_id(&self) -> Uuid;
}

impl TransactionEventSet for () {
    fn event_set_id(&self) -> Uuid {
        Uuid::nil()
    }
}

#[automock(type Transaction = ();)]
#[async_trait]
pub trait TransactionManager: Send + Sync + 'static {
    // `Send` is required so the async_trait-generated futures are `Send`.
    type Transaction: Send + TransactionEventSet;

    async fn begin(
        &self,
        user_id: &UserId,
        operation: EventSetOperation,
    ) -> Result<Self::Transaction, DomainError>;

    async fn commit(&self, tx: Self::Transaction) -> Result<(), DomainError>;
}
