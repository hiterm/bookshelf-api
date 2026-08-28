#![cfg(test)]

use anyhow::{Context, Result};
use bookshelf_e2e::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn operation_detail_groups_merge_changes() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;
    let source = create_test_author("Operation Merge Source", &token).await?;
    let destination = create_test_author("Operation Merge Destination", &token).await?;
    let book = create_test_book("Operation Merge Book", &source, &token).await?;
    let mutation = format!(
        r#"mutation {{ mergeAuthor(sourceAuthorId: "{source}", destinationAuthorId: "{destination}") {{ operationId }} }}"#
    );
    let (_, response) = graphql_request(&mutation, Some(&token)).await?;
    assert_no_graphql_errors(&response, "mergeAuthor");
    let operation_id = response["data"]["mergeAuthor"]["operationId"]
        .as_str()
        .context("merge operationId")?;

    let query = format!(
        r#"{{ operation(id: "{operation_id}") {{
          type detail
          bookChanges {{ bookId beforeRevision {{ revisionNumber }} afterRevision {{ revisionNumber }} }}
          authorChanges {{ authorId beforeRevision {{ revisionNumber }} afterRevision {{ revisionNumber }} }}
        }} }}"#
    );
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    assert_no_graphql_errors(&response, "merge Operation detail");
    let operation = &response["data"]["operation"];
    assert_eq!(operation["type"], "merge_author");
    assert_eq!(operation["bookChanges"].as_array().map(Vec::len), Some(1));
    assert_eq!(operation["authorChanges"].as_array().map(Vec::len), Some(2));

    delete_test_book(&book, &token).await?;
    delete_test_author(&destination, &token).await?;
    Ok(())
}
