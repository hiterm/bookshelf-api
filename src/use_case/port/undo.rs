use async_trait::async_trait;
use mockall::automock;

use crate::domain::{
    entity::{operation::OperationId, user::UserId},
    error::DomainError,
};

/// Executes one owned Undo as an indivisible command.
///
/// Implementations must lock and revalidate the target and affected entities,
/// then apply inverse state and record the new Operation, Revisions and changes
/// in a single transaction. Return the new Operation ID only after commit.
/// Any failure must roll back all writes. Eligibility reads are advisory and
/// must never replace validation inside this command's transaction.
#[automock]
#[async_trait]
pub trait UndoCommand: Send + Sync + 'static {
    async fn undo_operation(
        &self,
        user_id: &UserId,
        operation_id: &OperationId,
    ) -> Result<OperationId, DomainError>;
}
