// E2E tests that run against a real Postgres instance.

#![cfg(test)]

use anyhow::{Context, Result};
use bookshelf_e2e::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn e2e_book_events_records_create_operation() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let author_id =
        create_test_author(&format!("History Author {}", uuid::Uuid::new_v4()), &token).await?;
    let book_id = create_test_book("History Book Create", &author_id, &token).await?;

    let query = format!(
        r#"{{ bookEvents(bookId: "{}") {{
            eventId eventSetId operation bookId
            title authorIds isbn read owned priority format store
            bookCreatedAt bookUpdatedAt changedAt extra
        }} }}"#,
        book_id
    );
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    assert_no_graphql_errors(&response, "bookEvents");

    let entries = response["data"]["bookEvents"]
        .as_array()
        .context("bookEvents should be an array")?;
    assert_eq!(entries.len(), 1, "should have exactly 1 event entry");

    let entry = &entries[0];
    assert!(!entry["eventId"].is_null(), "eventId should not be null");
    assert!(
        !entry["eventSetId"].is_null(),
        "eventSetId should not be null"
    );
    assert_eq!(entry["operation"].as_str(), Some("create"));
    assert_eq!(
        entry["bookId"].as_str(),
        Some(book_id.as_str()),
        "bookId should match"
    );
    assert_eq!(entry["title"].as_str(), Some("History Book Create"));
    assert_eq!(
        entry["authorIds"].as_array().and_then(|a| a[0].as_str()),
        Some(author_id.as_str()),
        "authorIds should contain the author"
    );
    assert_eq!(
        entry["isbn"].as_str(),
        Some(""),
        "isbn should be empty string"
    );
    assert_eq!(entry["read"].as_bool(), Some(false), "read should be false");
    assert_eq!(
        entry["owned"].as_bool(),
        Some(false),
        "owned should be false"
    );
    assert_eq!(
        entry["priority"].as_i64(),
        Some(50),
        "priority should be 50"
    );
    assert_eq!(
        entry["format"].as_str(),
        Some("E_BOOK"),
        "format should be E_BOOK"
    );
    assert_eq!(
        entry["store"].as_str(),
        Some("KINDLE"),
        "store should be KINDLE"
    );
    assert!(
        !entry["bookCreatedAt"].is_null(),
        "bookCreatedAt should not be null"
    );
    assert!(
        !entry["bookUpdatedAt"].is_null(),
        "bookUpdatedAt should not be null"
    );
    assert!(
        !entry["changedAt"].is_null(),
        "changedAt should not be null"
    );
    assert!(
        entry["extra"].is_null(),
        "create event extra should be null"
    );

    // Cleanup
    delete_test_book(&book_id, &token).await?;
    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_book_events_records_update_operation() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let author_id =
        create_test_author(&format!("History Author {}", uuid::Uuid::new_v4()), &token).await?;
    let book_id = create_test_book("Original Title", &author_id, &token).await?;

    // Update the book
    update_test_book(
        &book_id,
        "Updated Title",
        &author_id,
        "",
        false,
        false,
        50,
        "E_BOOK",
        "KINDLE",
        &token,
    )
    .await?;

    let query = format!(
        r#"{{ bookEvents(bookId: "{}") {{
            eventId eventSetId operation bookId
            title authorIds isbn read owned priority format store
            bookCreatedAt bookUpdatedAt changedAt extra
        }} }}"#,
        book_id
    );
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    let entries = response["data"]["bookEvents"]
        .as_array()
        .context("bookEvents should be an array")?;
    assert_eq!(
        entries.len(),
        2,
        "should have 2 event entries (create + update)"
    );

    // Entries are ordered by changed_at DESC, so the most recent (update) is first.
    // Each entry records the post-state after the operation was applied.
    let update_entry = &entries[0];
    assert_eq!(update_entry["operation"].as_str(), Some("update"));
    assert!(
        !update_entry["eventId"].is_null(),
        "update eventId should not be null"
    );
    assert!(
        !update_entry["eventSetId"].is_null(),
        "update eventSetId should not be null"
    );
    assert_eq!(update_entry["bookId"].as_str(), Some(book_id.as_str()));
    assert_eq!(update_entry["title"].as_str(), Some("Updated Title"));
    assert_eq!(
        update_entry["authorIds"]
            .as_array()
            .and_then(|a| a[0].as_str()),
        Some(author_id.as_str()),
    );
    assert_eq!(update_entry["isbn"].as_str(), Some(""));
    assert_eq!(update_entry["read"].as_bool(), Some(false));
    assert_eq!(update_entry["owned"].as_bool(), Some(false));
    assert_eq!(update_entry["priority"].as_i64(), Some(50));
    assert_eq!(update_entry["format"].as_str(), Some("E_BOOK"));
    assert_eq!(update_entry["store"].as_str(), Some("KINDLE"));
    assert!(!update_entry["bookCreatedAt"].is_null());
    assert!(!update_entry["bookUpdatedAt"].is_null());
    assert!(!update_entry["changedAt"].is_null());
    assert!(
        update_entry["extra"].is_null(),
        "update event extra should be null"
    );

    let create_entry = &entries[1];
    assert_eq!(create_entry["operation"].as_str(), Some("create"));
    assert_eq!(create_entry["title"].as_str(), Some("Original Title"));

    // Cleanup
    delete_test_book(&book_id, &token).await?;
    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_book_events_records_delete_operation() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let author_id =
        create_test_author(&format!("History Author {}", uuid::Uuid::new_v4()), &token).await?;
    let book_id = create_test_book("Book To Delete", &author_id, &token).await?;

    // Grab history before delete so we know the book_id for later query
    let history_query = format!(
        r#"{{ bookEvents(bookId: "{}") {{ eventId operation title }} }}"#,
        book_id
    );
    let (_, response) = graphql_request(&history_query, Some(&token)).await?;
    let entries_before = response["data"]["bookEvents"]
        .as_array()
        .context("bookEvents should be an array")?;
    assert_eq!(
        entries_before.len(),
        1,
        "should have 1 history entry before delete"
    );

    // Delete the book
    delete_test_book(&book_id, &token).await?;

    // Even after deletion, history should be queryable and include the delete entry
    let (_, response) = graphql_request(&history_query, Some(&token)).await?;
    let entries_after = response["data"]["bookEvents"]
        .as_array()
        .context("bookEvents should be an array")?;
    assert_eq!(
        entries_after.len(),
        2,
        "should have 2 history entries after delete (create + delete)"
    );
    assert_eq!(
        entries_after[0]["operation"].as_str(),
        Some("delete"),
        "most recent entry should be 'delete'"
    );

    // Cleanup author
    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_author_events_records_create_operation() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let author_name = format!("Author History Create {}", uuid::Uuid::new_v4());
    let author_id = create_test_author(&author_name, &token).await?;

    let query = format!(
        r#"{{ authorEvents(authorId: "{}") {{
            eventId eventSetId operation authorId
            name yomi authorCreatedAt authorUpdatedAt changedAt extra
        }} }}"#,
        author_id
    );
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    assert_no_graphql_errors(&response, "authorEvents");

    let entries = response["data"]["authorEvents"]
        .as_array()
        .context("authorEvents should be an array")?;
    assert_eq!(entries.len(), 1, "should have exactly 1 event entry");

    let entry = &entries[0];
    assert!(!entry["eventId"].is_null(), "eventId should not be null");
    assert!(
        !entry["eventSetId"].is_null(),
        "eventSetId should not be null"
    );
    assert_eq!(entry["operation"].as_str(), Some("create"));
    assert_eq!(
        entry["authorId"].as_str(),
        Some(author_id.as_str()),
        "authorId should match"
    );
    assert_eq!(entry["name"].as_str(), Some(author_name.as_str()));
    assert_eq!(
        entry["yomi"].as_str(),
        Some(""),
        "yomi defaults to empty string"
    );
    assert!(
        !entry["authorCreatedAt"].is_null(),
        "authorCreatedAt should not be null"
    );
    assert!(
        !entry["authorUpdatedAt"].is_null(),
        "authorUpdatedAt should not be null"
    );
    assert!(
        !entry["changedAt"].is_null(),
        "changedAt should not be null"
    );
    assert!(
        entry["extra"].is_null(),
        "create event extra should be null"
    );

    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_author_events_records_update_operation() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let original_name = format!("Author Before Update History {}", uuid::Uuid::new_v4());
    let author_id = create_test_author(&original_name, &token).await?;

    let updated_name = format!("Author After Update History {}", uuid::Uuid::new_v4());
    let update_query = format!(
        r#"mutation {{ updateAuthor(authorData: {{ id: "{}", name: "{}" }}) {{ id name }} }}"#,
        author_id, updated_name
    );
    let (_, response) = graphql_request(&update_query, Some(&token)).await?;
    assert_no_graphql_errors(&response, "updateAuthor");

    let query = format!(
        r#"{{ authorEvents(authorId: "{}") {{
            eventId eventSetId operation authorId
            name yomi authorCreatedAt authorUpdatedAt changedAt extra
        }} }}"#,
        author_id
    );
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    let entries = response["data"]["authorEvents"]
        .as_array()
        .context("authorEvents should be an array")?;
    assert_eq!(
        entries.len(),
        2,
        "should have 2 event entries (create + update)"
    );

    // Each entry records the post-state after the operation was applied.
    let update_entry = &entries[0];
    assert_eq!(update_entry["operation"].as_str(), Some("update"));
    assert!(!update_entry["eventId"].is_null());
    assert!(!update_entry["eventSetId"].is_null());
    assert_eq!(update_entry["authorId"].as_str(), Some(author_id.as_str()));
    assert_eq!(update_entry["name"].as_str(), Some(updated_name.as_str()));
    assert_eq!(
        update_entry["yomi"].as_str(),
        Some(""),
        "yomi should remain empty string"
    );
    assert!(!update_entry["authorCreatedAt"].is_null());
    assert!(!update_entry["authorUpdatedAt"].is_null());
    assert!(!update_entry["changedAt"].is_null());
    assert!(
        update_entry["extra"].is_null(),
        "update event extra should be null"
    );

    let create_entry = &entries[1];
    assert_eq!(create_entry["operation"].as_str(), Some("create"));
    assert_eq!(create_entry["name"].as_str(), Some(original_name.as_str()));

    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_author_events_records_delete_operation() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let author_name = format!("Author History Delete {}", uuid::Uuid::new_v4());
    let author_id = create_test_author(&author_name, &token).await?;

    // Confirm 1 history entry before delete
    let history_query = format!(
        r#"{{ authorEvents(authorId: "{}") {{ eventId operation name }} }}"#,
        author_id
    );
    let (_, response) = graphql_request(&history_query, Some(&token)).await?;
    let before = response["data"]["authorEvents"]
        .as_array()
        .context("authorEvents should be an array")?;
    assert_eq!(before.len(), 1, "should have 1 history entry before delete");

    // Delete the author
    delete_test_author(&author_id, &token).await?;

    // History should survive deletion and include the delete entry
    let (_, response) = graphql_request(&history_query, Some(&token)).await?;
    let after = response["data"]["authorEvents"]
        .as_array()
        .context("authorEvents should be an array")?;
    assert_eq!(
        after.len(),
        2,
        "should have 2 history entries after delete (create + delete)"
    );
    assert_eq!(
        after[0]["operation"].as_str(),
        Some("delete"),
        "most recent entry should be 'delete'"
    );
    assert!(
        after[0]["name"].is_null(),
        "delete event records only author_id; name is null"
    );
    Ok(())
}
