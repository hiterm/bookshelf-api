use async_trait::async_trait;
use mockall::automock;

use uuid::Uuid;

use crate::domain::{
    entity::{
        operation::{NewOperation, OperationId, OperationType},
        user::UserId,
    },
    error::DomainError,
};

pub trait TransactionOperation {
    fn operation_id(&self) -> OperationId;
    fn revision_number(&self) -> Option<i32>;
}

impl TransactionOperation for () {
    fn operation_id(&self) -> OperationId {
        OperationId::from(Uuid::nil())
    }

    fn revision_number(&self) -> Option<i32> {
        Some(1)
    }
}

#[automock(type Transaction = ();)]
#[async_trait]
pub trait TransactionManager: Send + Sync + 'static {
    // `Send` is required so the async_trait-generated futures are `Send`.
    type Transaction: Send + TransactionOperation;

    async fn begin(
        &self,
        user_id: &UserId,
        operation_type: OperationType,
    ) -> Result<Self::Transaction, DomainError>;

    async fn begin_operation(
        &self,
        user_id: &UserId,
        operation: &NewOperation,
    ) -> Result<Self::Transaction, DomainError>;

    async fn commit(&self, tx: Self::Transaction) -> Result<(), DomainError>;

    async fn rollback(&self, tx: Self::Transaction) -> Result<(), DomainError>;
}
