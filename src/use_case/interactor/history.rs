use std::collections::HashMap;

use async_trait::async_trait;

use crate::{
    domain::{
        entity::{
            author::AuthorId,
            book::BookId,
            operation::OperationId,
            revision::{AuthorRevisionKey, BookRevisionKey, RevisionNumber},
            user::UserId,
        },
        repository::history_repository::HistoryRepository,
    },
    use_case::{
        dto::history::{
            AuthorOperationChangeDto, AuthorRevisionDto, AuthorRevisionKeyDto,
            BookOperationChangeDto, BookRevisionDto, BookRevisionKeyDto, OperationDto,
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

    async fn book_revisions_by_keys(
        &self,
        user_id: &str,
        keys: &[BookRevisionKeyDto],
    ) -> Result<HashMap<BookRevisionKeyDto, BookRevisionDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_owned())?;
        let keys = keys
            .iter()
            .map(|key| {
                Ok(BookRevisionKey {
                    book_id: BookId::try_from(key.book_id.as_str())?,
                    revision_number: RevisionNumber::try_from(key.revision_number)?,
                })
            })
            .collect::<Result<Vec<_>, UseCaseError>>()?;
        Ok(self
            .repository
            .find_book_revisions_by_keys(&user_id, &keys)
            .await?
            .into_iter()
            .map(|(key, revision)| {
                (
                    BookRevisionKeyDto {
                        book_id: key.book_id.to_string(),
                        revision_number: key.revision_number.value(),
                    },
                    revision.into(),
                )
            })
            .collect())
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

    async fn author_revisions_by_keys(
        &self,
        user_id: &str,
        keys: &[AuthorRevisionKeyDto],
    ) -> Result<HashMap<AuthorRevisionKeyDto, AuthorRevisionDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_owned())?;
        let keys = keys
            .iter()
            .map(|key| {
                Ok(AuthorRevisionKey {
                    author_id: AuthorId::try_from(key.author_id.as_str())?,
                    revision_number: RevisionNumber::try_from(key.revision_number)?,
                })
            })
            .collect::<Result<Vec<_>, UseCaseError>>()?;
        Ok(self
            .repository
            .find_author_revisions_by_keys(&user_id, &keys)
            .await?
            .into_iter()
            .map(|(key, revision)| {
                (
                    AuthorRevisionKeyDto {
                        author_id: key.author_id.to_string(),
                        revision_number: key.revision_number.value(),
                    },
                    revision.into(),
                )
            })
            .collect())
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
            entity::{
                book::BookId,
                operation::OperationId,
                revision::{BookRevisionKey, RevisionNumber},
            },
            repository::history_repository::MockHistoryRepository,
        },
        use_case::{
            dto::history::BookRevisionKeyDto,
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

    #[tokio::test]
    async fn book_revision_keys_are_validated_and_sent_in_one_batch() {
        let first_id = Uuid::new_v4();
        let second_id = Uuid::new_v4();
        let expected_keys = vec![
            BookRevisionKey {
                book_id: BookId::new(first_id).unwrap(),
                revision_number: RevisionNumber::FIRST,
            },
            BookRevisionKey {
                book_id: BookId::new(second_id).unwrap(),
                revision_number: RevisionNumber::try_from(2).unwrap(),
            },
        ];
        let expected_for_match = expected_keys.clone();
        let mut repository = MockHistoryRepository::new();
        repository
            .expect_find_book_revisions_by_keys()
            .withf(move |user_id, keys| user_id.as_str() == "user1" && keys == expected_for_match)
            .times(1)
            .return_once(|_, _| Ok(HashMap::new()));
        let keys = vec![
            BookRevisionKeyDto {
                book_id: first_id.to_string(),
                revision_number: 1,
            },
            BookRevisionKeyDto {
                book_id: second_id.to_string(),
                revision_number: 2,
            },
        ];

        let result = HistoryQueryInteractor::new(repository)
            .book_revisions_by_keys("user1", &keys)
            .await
            .unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn revision_batch_rejects_invalid_key_before_repository_call() {
        let repository = MockHistoryRepository::new();
        let result = HistoryQueryInteractor::new(repository)
            .book_revisions_by_keys(
                "user1",
                &[BookRevisionKeyDto {
                    book_id: Uuid::new_v4().to_string(),
                    revision_number: 0,
                }],
            )
            .await;

        assert!(result.is_err());
    }
}
