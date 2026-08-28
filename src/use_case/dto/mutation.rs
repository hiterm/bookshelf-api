use super::{author::AuthorDto, book::BookDto};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutationResultDto<T> {
    pub value: T,
    pub operation_id: String,
}

impl<T> MutationResultDto<T> {
    pub fn new(value: T, operation_id: String) -> Self {
        Self {
            value,
            operation_id,
        }
    }
}

/// Result of a mutation that produces exactly one event for the mutated entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SingleEventMutationResultDto<T> {
    pub value: T,
    pub operation_id: String,
    pub revision_number: i32,
}

impl<T> SingleEventMutationResultDto<T> {
    pub fn new(value: T, operation_id: String, revision_number: i32) -> Self {
        Self {
            value,
            operation_id,
            revision_number,
        }
    }
}

pub type BookMutationResultDto = SingleEventMutationResultDto<BookDto>;
pub type AuthorMutationResultDto = SingleEventMutationResultDto<AuthorDto>;
pub type DeleteBookResultDto = MutationResultDto<String>;
pub type DeleteAuthorResultDto = MutationResultDto<String>;
pub type ImportBooksResultDto = MutationResultDto<Vec<BookDto>>;
pub type RestoreBookResultDto = SingleEventMutationResultDto<Option<BookDto>>;
pub type RestoreAuthorResultDto = SingleEventMutationResultDto<Option<AuthorDto>>;
