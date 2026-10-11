#![cfg(test)]

use anyhow::{Context, Result};
use bookshelf_e2e::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn restore_book_uses_owned_revision_and_appends_a_revision() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;
    let author_id = create_test_author("Revision Restore Author", &token).await?;
    let (book_id, source_revision, _) =
        create_test_book_with_event("Before Restore", &author_id, &token).await?;
    let update = format!(
        r#"mutation {{ updateBook(bookData: {{
          id: "{book_id}", title: "After Update", authorIds: ["{author_id}"],
          isbn: "", read: true, owned: false, priority: 50,
          format: E_BOOK, store: KINDLE
        }}) {{ revisionNumber }} }}"#
    );
    graphql_request(&update, Some(&token)).await?;

    let restore = format!(
        r#"mutation {{ restoreBook(bookId: "{book_id}", revisionNumber: {source_revision}) {{
          operationId revisionNumber book {{ id title read }}
        }} }}"#
    );
    let (_, response) = graphql_request(&restore, Some(&token)).await?;
    assert_no_graphql_errors(&response, "restoreBook by revision");
    let payload = &response["data"]["restoreBook"];
    assert_eq!(payload["book"]["title"], "Before Restore");
    assert_eq!(payload["book"]["read"], false);
    assert_eq!(payload["revisionNumber"], 3);
    payload["operationId"]
        .as_str()
        .context("restore operationId should be a string")?;

    delete_test_book(&book_id, &token).await?;
    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn restore_author_uses_revision_and_is_tenant_scoped() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;
    let (author_id, source_revision, _) =
        create_test_author_with_event("Before Author Restore", &token).await?;
    let update = format!(
        r#"mutation {{ updateAuthor(authorData: {{ id: "{author_id}", name: "After Author Update" }}) {{ revisionNumber }} }}"#
    );
    graphql_request(&update, Some(&token)).await?;

    let restore = format!(
        r#"mutation {{ restoreAuthor(authorId: "{author_id}", revisionNumber: {source_revision}) {{
          operationId revisionNumber author {{ id name }}
        }} }}"#
    );
    let (_, response) = graphql_request(&restore, Some(&token)).await?;
    assert_no_graphql_errors(&response, "restoreAuthor by revision");
    assert_eq!(
        response["data"]["restoreAuthor"]["author"]["name"],
        "Before Author Restore"
    );
    assert_eq!(response["data"]["restoreAuthor"]["revisionNumber"], 3);

    let (_other_user, other_token) = create_test_user().await?;
    let (_, response) = graphql_request(&restore, Some(&other_token)).await?;
    assert_graphql_error(
        &response,
        "cross-tenant restoreAuthor",
        "NOT_FOUND",
        &["restoreAuthor"],
    );

    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn restore_rejects_invalid_or_missing_revision() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;
    let missing_id = uuid::Uuid::new_v4();
    for (query, code, path) in [
        (
            format!(
                r#"mutation {{ restoreBook(bookId: "{missing_id}", revisionNumber: 0) {{ operationId }} }}"#
            ),
            "VALIDATION_ERROR",
            "restoreBook",
        ),
        (
            format!(
                r#"mutation {{ restoreAuthor(authorId: "{missing_id}", revisionNumber: 999) {{ operationId }} }}"#
            ),
            "NOT_FOUND",
            "restoreAuthor",
        ),
    ] {
        let (_, response) = graphql_request(&query, Some(&token)).await?;
        assert_graphql_error(
            &response,
            "invalid or missing revision restore",
            code,
            &[path],
        );
    }
    Ok(())
}

#[tokio::test]
#[serial]
async fn restore_and_undo_author_name_conflicts_are_actionable_and_recoverable() -> Result<()> {
    let (_, token) = create_test_user().await?;
    let (id, revision, _) = create_test_author_with_event("Conflict Original", &token).await?;
    let update = format!(
        r#"mutation {{ updateAuthor(authorData: {{ id: "{id}", name: "Conflict Changed" }}) {{ operationId }} }}"#
    );
    let (_, response) = graphql_request(&update, Some(&token)).await?;
    assert_no_graphql_errors(&response, "rename before conflict");
    let operation = response["data"]["updateAuthor"]["operationId"]
        .as_str()
        .context("update operation")?;
    let blocking_id = create_test_author("Conflict Original", &token).await?;
    let state_query = format!(
        r#"{{ author(id: "{id}") {{ id name yomi createdAt updatedAt }} authors {{ id name yomi createdAt updatedAt }} authorRevisions(authorId: "{id}") {{ revisionNumber name yomi }} operations {{ id type }} }}"#
    );
    let (_, before) = graphql_request(&state_query, Some(&token)).await?;
    assert_no_graphql_errors(&before, "state before conflict");
    let restore = format!(
        r#"mutation {{ restoreAuthor(authorId: "{id}", revisionNumber: {revision}) {{ operationId revisionNumber author {{ name }} }} }}"#
    );
    let undo =
        format!(r#"mutation {{ undoOperation(operationId: "{operation}") {{ operationId }} }}"#);
    for (request, field) in [(&restore, "restoreAuthor"), (&undo, "undoOperation")] {
        let (_, response) = graphql_request(request, Some(&token)).await?;
        assert_graphql_error(&response, "author name conflict", "CONFLICT", &[field]);
        assert_eq!(
            response["errors"][0]["extensions"]["reason"],
            "AUTHOR_NAME_CONFLICT"
        );
        assert_eq!(
            response["errors"][0]["message"],
            "author name 'Conflict Original' is already in use"
        );
        let (_, after) = graphql_request(&state_query, Some(&token)).await?;
        assert_no_graphql_errors(&after, "state after conflict");
        assert_eq!(after["data"], before["data"]);
    }
    delete_test_author(&blocking_id, &token).await?;
    // Undo succeeds once the name is available. Restore remains valid afterward.
    let (_, response) = graphql_request(&undo, Some(&token)).await?;
    assert_no_graphql_errors(&response, "retry undo after resolving conflict");
    let (_, response) = graphql_request(&restore, Some(&token)).await?;
    assert_no_graphql_errors(&response, "retry restore after resolving conflict");
    assert_eq!(
        response["data"]["restoreAuthor"]["author"]["name"],
        "Conflict Original"
    );
    assert_eq!(response["data"]["restoreAuthor"]["revisionNumber"], 4);
    delete_test_author(&id, &token).await?;
    Ok(())
}
