use super::{author::AuthorDto, book::BookDto};
use crate::domain::entity::event::EventId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutationResultDto<T> {
    pub value: T,
    pub event_set_id: String,
}

impl<T> MutationResultDto<T> {
    pub fn new(value: T, event_set_id: String) -> Self {
        Self {
            value,
            event_set_id,
        }
    }
}

impl<T> std::ops::Deref for MutationResultDto<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// Result of a mutation that produces exactly one event for the mutated entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SingleEventMutationResultDto<T> {
    pub value: T,
    pub event_set_id: String,
    pub event_id: EventId,
}

impl<T> SingleEventMutationResultDto<T> {
    pub fn new(value: T, event_set_id: String, event_id: EventId) -> Self {
        Self {
            value,
            event_set_id,
            event_id,
        }
    }
}

impl<T> std::ops::Deref for SingleEventMutationResultDto<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

pub type BookMutationResultDto = SingleEventMutationResultDto<BookDto>;
pub type AuthorMutationResultDto = SingleEventMutationResultDto<AuthorDto>;
pub type DeleteBookResultDto = MutationResultDto<String>;
pub type DeleteAuthorResultDto = MutationResultDto<String>;
pub type ImportBooksResultDto = MutationResultDto<Vec<BookDto>>;
pub type RestoreBookResultDto = MutationResultDto<Option<BookDto>>;
pub type RestoreAuthorResultDto = MutationResultDto<Option<AuthorDto>>;
