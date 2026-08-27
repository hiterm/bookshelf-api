#![cfg(test)]

use anyhow::{Context, Result};
use bookshelf_e2e::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn operations_expose_owned_batched_changes_and_revisions() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;
    let author_id = create_test_author("Operation History Author", &token).await?;
    let (book_id, revision_number, operation_id) =
        create_test_book_with_event("Operation History Book", &author_id, &token).await?;

    let query = format!(
        r#"{{
          operation(id: "{operation_id}") {{
            id type
            bookChanges {{
              bookId
              beforeRevision {{ revisionNumber }}
              afterRevision {{ revisionNumber title authorIds }}
            }}
          }}
          bookRevisions(bookId: "{book_id}") {{ revisionNumber title }}
          bookRevision(bookId: "{book_id}", revisionNumber: {revision_number}) {{ title }}
        }}"#
    );
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    assert_no_graphql_errors(&response, "owned Operation history");
    let operation = &response["data"]["operation"];
    assert_eq!(operation["id"].as_str(), Some(operation_id.as_str()));
    assert_eq!(operation["type"].as_str(), Some("create_book"));
    let changes = operation["bookChanges"]
        .as_array()
        .context("bookChanges should be an array")?;
    assert_eq!(changes.len(), 1);
    assert!(changes[0]["beforeRevision"].is_null());
    assert_eq!(changes[0]["afterRevision"]["revisionNumber"], 1);
    assert_eq!(response["data"]["bookRevisions"][0]["revisionNumber"], 1);
    assert_eq!(
        response["data"]["bookRevision"]["title"],
        "Operation History Book"
    );

    let (_other_user, other_token) = create_test_user().await?;
    let query = format!(r#"{{ operation(id: "{operation_id}") {{ id }} }}"#);
    let (_, response) = graphql_request(&query, Some(&other_token)).await?;
    assert_no_graphql_errors(&response, "cross-tenant Operation lookup");
    assert!(response["data"]["operation"].is_null());

    delete_test_book(&book_id, &token).await?;
    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn operations_list_hides_baseline_and_loads_multiple_change_groups() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;
    let author_id = create_test_author("Operation List Author", &token).await?;
    let book_one = create_test_book("Operation List One", &author_id, &token).await?;
    let book_two = create_test_book("Operation List Two", &author_id, &token).await?;

    let (_, response) = graphql_request(
        "{ operations { id type bookChanges { bookId } authorChanges { authorId } } }",
        Some(&token),
    )
    .await?;
    assert_no_graphql_errors(&response, "operations list");
    let operations = response["data"]["operations"]
        .as_array()
        .context("operations should be an array")?;
    assert!(
        operations
            .iter()
            .all(|operation| operation["type"] != "baseline")
    );
    assert!(operations.len() >= 3);

    delete_test_book(&book_one, &token).await?;
    delete_test_book(&book_two, &token).await?;
    delete_test_author(&author_id, &token).await?;
    Ok(())
}
