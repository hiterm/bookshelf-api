use std::collections::HashMap;

use async_trait::async_trait;

use crate::{
    domain::{
        entity::{
            author::AuthorId, book::BookId, operation::OperationId, revision::RevisionNumber,
            user::UserId,
        },
        repository::history_repository::HistoryRepository,
    },
    use_case::{
        dto::history::{
            AuthorOperationChangeDto, AuthorRevisionDto, BookOperationChangeDto, BookRevisionDto,
            OperationDto,
        },
        error::UseCaseError,
        traits::history::{HistoryCommandUseCase, HistoryQueryUseCase},
    },
};

#[derive(Debug, Clone)]
pub struct HistoryQueryInteractor<HR> {
    repository: HR,
}

#[async_trait]
impl<HR: HistoryRepository> HistoryCommandUseCase for HistoryQueryInteractor<HR> {
    async fn undo_operation(
        &self,
        user_id: &str,
        operation_id: &str,
    ) -> Result<String, UseCaseError> {
        let user_id = UserId::new(user_id.to_owned())?;
        let operation_id = OperationId::try_from(operation_id).map_err(UseCaseError::Validation)?;
        Ok(self
            .repository
            .undo_operation(&user_id, &operation_id)
            .await?
            .to_string())
    }
}

impl<HR> HistoryQueryInteractor<HR> {
    pub fn new(repository: HR) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<HR: HistoryRepository> HistoryQueryUseCase for HistoryQueryInteractor<HR> {
    async fn operations(&self, user_id: &str) -> Result<Vec<OperationDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_owned())?;
        Ok(self
            .repository
            .find_operations(&user_id)
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    async fn operation(
        &self,
        user_id: &str,
        operation_id: &str,
    ) -> Result<Option<OperationDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_owned())?;
        let operation_id = OperationId::try_from(operation_id).map_err(UseCaseError::Validation)?;
        Ok(self
            .repository
            .find_operation(&user_id, &operation_id)
            .await?
            .map(Into::into))
    }

    async fn is_operation_undoable(
        &self,
        user_id: &str,
        operation_id: &str,
    ) -> Result<bool, UseCaseError> {
        let user_id = UserId::new(user_id.to_owned())?;
        let operation_id = OperationId::try_from(operation_id).map_err(UseCaseError::Validation)?;
        Ok(self
            .repository
            .is_operation_undoable(&user_id, &operation_id)
            .await?)
    }

    async fn book_revisions(
        &self,
        user_id: &str,
        book_id: &str,
    ) -> Result<Vec<BookRevisionDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_owned())?;
        let book_id = BookId::try_from(book_id)?;
        Ok(self
            .repository
            .find_book_revisions(&user_id, &book_id)
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    async fn book_revision(
        &self,
        user_id: &str,
        book_id: &str,
        revision_number: i32,
    ) -> Result<Option<BookRevisionDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_owned())?;
        let book_id = BookId::try_from(book_id)?;
        let revision_number = RevisionNumber::try_from(revision_number)?;
        Ok(self
            .repository
            .find_book_revision(&user_id, &book_id, revision_number)
            .await?
            .map(Into::into))
    }

    async fn author_revisions(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<Vec<AuthorRevisionDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_owned())?;
        let author_id = AuthorId::try_from(author_id)?;
        Ok(self
            .repository
            .find_author_revisions(&user_id, &author_id)
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    async fn author_revision(
        &self,
        user_id: &str,
        author_id: &str,
        revision_number: i32,
    ) -> Result<Option<AuthorRevisionDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_owned())?;
        let author_id = AuthorId::try_from(author_id)?;
        let revision_number = RevisionNumber::try_from(revision_number)?;
        Ok(self
            .repository
            .find_author_revision(&user_id, &author_id, revision_number)
            .await?
            .map(Into::into))
    }

    async fn book_changes(
        &self,
        user_id: &str,
        operation_ids: &[String],
    ) -> Result<HashMap<String, Vec<BookOperationChangeDto>>, UseCaseError> {
        let user_id = UserId::new(user_id.to_owned())?;
        let ids = parse_operation_ids(operation_ids)?;
        Ok(self
            .repository
            .find_book_changes_by_operation_ids(&user_id, &ids)
            .await?
            .into_iter()
            .map(|(id, changes)| {
                (
                    id.to_string(),
                    changes.into_iter().map(Into::into).collect(),
                )
            })
            .collect())
    }

    async fn author_changes(
        &self,
        user_id: &str,
        operation_ids: &[String],
    ) -> Result<HashMap<String, Vec<AuthorOperationChangeDto>>, UseCaseError> {
        let user_id = UserId::new(user_id.to_owned())?;
        let ids = parse_operation_ids(operation_ids)?;
        Ok(self
            .repository
            .find_author_changes_by_operation_ids(&user_id, &ids)
            .await?
            .into_iter()
            .map(|(id, changes)| {
                (
                    id.to_string(),
                    changes.into_iter().map(Into::into).collect(),
                )
            })
            .collect())
    }
}

fn parse_operation_ids(values: &[String]) -> Result<Vec<OperationId>, UseCaseError> {
    values
        .iter()
        .map(|value| OperationId::try_from(value.as_str()).map_err(UseCaseError::Validation))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use uuid::Uuid;

    use crate::{
        domain::{
            entity::operation::OperationId, repository::history_repository::MockHistoryRepository,
        },
        use_case::{
            interactor::history::HistoryQueryInteractor,
            traits::history::{HistoryCommandUseCase, HistoryQueryUseCase},
        },
    };

    #[tokio::test]
    async fn operation_rejects_invalid_id_before_repository_call() {
        let repository = MockHistoryRepository::new();
        let result = HistoryQueryInteractor::new(repository)
            .operation("user1", "invalid")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn undo_eligibility_is_scoped_to_the_parsed_owner_and_operation() {
        let operation_id = OperationId::from(Uuid::new_v4());
        let expected_id = operation_id.clone();
        let mut repository = MockHistoryRepository::new();
        repository
            .expect_is_operation_undoable()
            .withf(move |user_id, id| user_id.as_str() == "user1" && id == &expected_id)
            .return_once(|_, _| Ok(true));

        let result = HistoryQueryInteractor::new(repository)
            .is_operation_undoable("user1", &operation_id.to_string())
            .await
            .unwrap();

        assert!(result);
    }

    #[tokio::test]
    async fn undo_eligibility_rejects_invalid_id_before_repository_call() {
        let repository = MockHistoryRepository::new();

        let result = HistoryQueryInteractor::new(repository)
            .is_operation_undoable("user1", "invalid")
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn undo_returns_the_new_operation_id() {
        let target_id = OperationId::from(Uuid::new_v4());
        let undo_id = OperationId::from(Uuid::new_v4());
        let expected_target = target_id.clone();
        let expected_undo = undo_id.clone();
        let mut repository = MockHistoryRepository::new();
        repository
            .expect_undo_operation()
            .withf(move |user_id, id| user_id.as_str() == "user1" && id == &expected_target)
            .return_once(move |_, _| Ok(expected_undo));

        let result = HistoryQueryInteractor::new(repository)
            .undo_operation("user1", &target_id.to_string())
            .await
            .unwrap();

        assert_eq!(result, undo_id.to_string());
    }

    #[tokio::test]
    async fn batch_changes_preserve_empty_operation_entries() {
        let operation_id = OperationId::from(Uuid::new_v4());
        let expected = operation_id.clone();
        let expected_for_match = expected.clone();
        let mut repository = MockHistoryRepository::new();
        repository
            .expect_find_book_changes_by_operation_ids()
            .withf(move |_, ids| ids == [expected_for_match.clone()])
            .return_once(move |_, _| Ok(HashMap::from([(operation_id, vec![])])));
        let id = expected.to_string();
        let result = HistoryQueryInteractor::new(repository)
            .book_changes("user1", std::slice::from_ref(&id))
            .await
            .unwrap();
        assert_eq!(result[&id], vec![]);
    }

    #[tokio::test]
    async fn author_changes_parse_ids_and_preserve_empty_entries() {
        let operation_id = OperationId::from(Uuid::new_v4());
        let expected = operation_id.clone();
        let expected_for_match = expected.clone();
        let mut repository = MockHistoryRepository::new();
        repository
            .expect_find_author_changes_by_operation_ids()
            .withf(move |user_id, ids| {
                user_id.as_str() == "user1" && ids == [expected_for_match.clone()]
            })
            .return_once(move |_, _| Ok(HashMap::from([(operation_id, vec![])])));
        let id = expected.to_string();

        let result = HistoryQueryInteractor::new(repository)
            .author_changes("user1", std::slice::from_ref(&id))
            .await
            .unwrap();

        assert_eq!(result[&id], vec![]);
    }
}
