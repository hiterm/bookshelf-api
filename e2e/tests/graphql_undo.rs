#![cfg(test)]

use anyhow::{Context, Result};
use bookshelf_e2e::*;
use serial_test::serial;

async fn undo(operation_id: &str, token: &str) -> Result<serde_json::Value> {
    let mutation =
        format!(r#"mutation {{ undoOperation(operationId: "{operation_id}") {{ operationId }} }}"#);
    let (_, response) = graphql_request(&mutation, Some(token)).await?;
    Ok(response)
}

#[tokio::test]
#[serial]
async fn undo_create_and_undo_of_undo_round_trip_book_state() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;
    let author_id = create_test_author("Undo Author", &token).await?;
    let (book_id, _, create_operation_id) =
        create_test_book_with_event("Undo Book", &author_id, &token).await?;

    let response = undo(&create_operation_id, &token).await?;
    assert_no_graphql_errors(&response, "undo create Book");
    let undo_operation_id = response["data"]["undoOperation"]["operationId"]
        .as_str()
        .context("undo operation ID")?;

    let (_, response) = graphql_request(
        &format!(r#"{{ book(id: "{book_id}") {{ id }} }}"#),
        Some(&token),
    )
    .await?;
    assert_no_graphql_errors(&response, "query undone Book");
    assert!(response["data"]["book"].is_null());

    let response = undo(undo_operation_id, &token).await?;
    assert_no_graphql_errors(&response, "undo the undo");

    let (_, response) = graphql_request(
        &format!(
            r#"{{ book(id: "{book_id}") {{ id title }} bookRevisions(bookId: "{book_id}") {{ revisionNumber }} }}"#
        ),
        Some(&token),
    )
    .await?;
    assert_no_graphql_errors(&response, "query restored Book");
    assert_eq!(response["data"]["book"]["title"], "Undo Book");
    assert_eq!(response["data"]["bookRevisions"][0]["revisionNumber"], 2);

    delete_test_book(&book_id, &token).await?;
    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn undo_update_ignores_unrelated_changes_but_rejects_target_conflicts() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;
    let author_id = create_test_author("Undo Update Author", &token).await?;
    let (book_id, _, _) = create_test_book_with_event("Before", &author_id, &token).await?;
    let update = |title: &str| {
        format!(
            r#"mutation {{ updateBook(bookData: {{
              id: "{book_id}" title: "{title}" authorIds: ["{author_id}"]
              isbn: "" read: false owned: true priority: 1
              format: PRINTED store: UNKNOWN
            }}) {{ operationId }} }}"#
        )
    };
    let (_, response) = graphql_request(&update("First"), Some(&token)).await?;
    assert_no_graphql_errors(&response, "first update");
    let first_operation = response["data"]["updateBook"]["operationId"]
        .as_str()
        .context("first update operation")?
        .to_owned();

    let unrelated_author = create_test_author("Unrelated Undo Author", &token).await?;
    let response = undo(&first_operation, &token).await?;
    assert_no_graphql_errors(&response, "undo with unrelated later Operation");

    let (_, response) = graphql_request(&update("Second"), Some(&token)).await?;
    assert_no_graphql_errors(&response, "second update");
    let second_operation = response["data"]["updateBook"]["operationId"]
        .as_str()
        .context("second update operation")?
        .to_owned();
    let (_, response) = graphql_request(&update("Conflicting"), Some(&token)).await?;
    assert_no_graphql_errors(&response, "conflicting update");
    let response = undo(&second_operation, &token).await?;
    assert!(response.get("errors").is_some());

    delete_test_book(&book_id, &token).await?;
    delete_test_author(&author_id, &token).await?;
    delete_test_author(&unrelated_author, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn undo_delete_restores_the_entity_with_a_fresh_revision() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;
    let author_id = create_test_author("Undo Delete Author", &token).await?;
    let (book_id, _, _) = create_test_book_with_event("Deleted Book", &author_id, &token).await?;
    let mutation = format!(r#"mutation {{ deleteBook(bookId: "{book_id}") {{ operationId }} }}"#);
    let (_, response) = graphql_request(&mutation, Some(&token)).await?;
    assert_no_graphql_errors(&response, "delete before undo");
    let delete_operation = response["data"]["deleteBook"]["operationId"]
        .as_str()
        .context("delete operation")?;

    let response = undo(delete_operation, &token).await?;
    assert_no_graphql_errors(&response, "undo delete");
    let (_, response) = graphql_request(
        &format!(r#"{{ book(id: "{book_id}") {{ title }} bookRevisions(bookId: "{book_id}") {{ revisionNumber }} }}"#),
        Some(&token),
    ).await?;
    assert_eq!(response["data"]["book"]["title"], "Deleted Book");
    assert_eq!(response["data"]["bookRevisions"][0]["revisionNumber"], 2);

    delete_test_book(&book_id, &token).await?;
    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn undo_import_removes_created_books_and_authors_atomically() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;
    let author_name = format!("Imported Undo Author {}", uuid::Uuid::new_v4());
    let mutation = format!(
        r#"mutation {{ importBooks(books: [{{
          title: "Imported Undo Book" authorNames: ["{author_name}"] isbn: ""
          read: false owned: true priority: 1 format: PRINTED store: UNKNOWN
        }}]) {{ operationId books {{ id }} }} }}"#
    );
    let (_, response) = graphql_request(&mutation, Some(&token)).await?;
    assert_no_graphql_errors(&response, "import before undo");
    let operation_id = response["data"]["importBooks"]["operationId"]
        .as_str()
        .context("import operation")?;
    let book_id = response["data"]["importBooks"]["books"][0]["id"]
        .as_str()
        .context("imported Book")?;

    let response = undo(operation_id, &token).await?;
    assert_no_graphql_errors(&response, "undo import");
    let (_, response) = graphql_request(
        &format!(r#"{{ book(id: "{book_id}") {{ id }} authors {{ name }} }}"#),
        Some(&token),
    )
    .await?;
    assert!(response["data"]["book"].is_null());
    assert!(
        response["data"]["authors"]
            .as_array()
            .context("authors")?
            .iter()
            .all(|author| author["name"] != author_name)
    );
    Ok(())
}

#[tokio::test]
#[serial]
async fn undo_merge_restores_source_and_book_relationships() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;
    let source_id = create_test_author("Undo Merge Source", &token).await?;
    let destination_id = create_test_author("Undo Merge Destination", &token).await?;
    let book_id = create_test_book("Undo Merge Book", &source_id, &token).await?;
    let mutation = format!(
        r#"mutation {{ mergeAuthor(sourceAuthorId: "{source_id}", destinationAuthorId: "{destination_id}") {{ operationId }} }}"#
    );
    let (_, response) = graphql_request(&mutation, Some(&token)).await?;
    assert_no_graphql_errors(&response, "merge before undo");
    let operation_id = response["data"]["mergeAuthor"]["operationId"]
        .as_str()
        .context("merge operation")?;

    let response = undo(operation_id, &token).await?;
    assert_no_graphql_errors(&response, "undo merge");
    let (_, response) = graphql_request(
        &format!(r#"{{ author(id: "{source_id}") {{ id }} book(id: "{book_id}") {{ authors {{ id }} }} }}"#),
        Some(&token),
    ).await?;
    assert_eq!(response["data"]["author"]["id"], source_id);
    assert_eq!(response["data"]["book"]["authors"][0]["id"], source_id);

    delete_test_book(&book_id, &token).await?;
    delete_test_author(&source_id, &token).await?;
    delete_test_author(&destination_id, &token).await?;
    Ok(())
}
