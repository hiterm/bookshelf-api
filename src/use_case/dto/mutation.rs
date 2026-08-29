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

/// Result of a mutation that produces exactly one revision for the entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SingleRevisionMutationResultDto<T> {
    pub value: T,
    pub operation_id: String,
    pub revision_number: i32,
}

impl<T> SingleRevisionMutationResultDto<T> {
    pub fn new(value: T, operation_id: String, revision_number: i32) -> Self {
        Self {
            value,
            operation_id,
            revision_number,
        }
    }
}

pub type BookMutationResultDto = SingleRevisionMutationResultDto<BookDto>;
pub type AuthorMutationResultDto = SingleRevisionMutationResultDto<AuthorDto>;
pub type DeleteBookResultDto = MutationResultDto<String>;
pub type DeleteAuthorResultDto = MutationResultDto<String>;
pub type ImportBooksResultDto = MutationResultDto<Vec<BookDto>>;
pub type RestoreBookResultDto = SingleRevisionMutationResultDto<Option<BookDto>>;
pub type RestoreAuthorResultDto = SingleRevisionMutationResultDto<Option<AuthorDto>>;
