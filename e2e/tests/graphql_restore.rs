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
    assert_graphql_errors(&response, "cross-tenant restoreAuthor");

    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn restore_rejects_invalid_or_missing_revision() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;
    let missing_id = uuid::Uuid::new_v4();
    for query in [
        format!(
            r#"mutation {{ restoreBook(bookId: "{missing_id}", revisionNumber: 0) {{ operationId }} }}"#
        ),
        format!(
            r#"mutation {{ restoreAuthor(authorId: "{missing_id}", revisionNumber: 999) {{ operationId }} }}"#
        ),
    ] {
        let (_, response) = graphql_request(&query, Some(&token)).await?;
        assert_graphql_errors(&response, "invalid or missing revision restore");
    }
    Ok(())
}
