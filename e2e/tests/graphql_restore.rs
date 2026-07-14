// E2E tests that run against a real Postgres instance.

#![cfg(test)]

use anyhow::{Context, Result};
use bookshelf_e2e::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn e2e_restore_book_reverts_to_snapshot() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let author_id =
        create_test_author(&format!("History Author {}", uuid::Uuid::new_v4()), &token).await?;
    let book_id = create_test_book("Before Restore", &author_id, &token).await?;

    // Update so history has 2 entries; record create event's event_id
    let history_query = format!(
        r#"{{ bookEvents(bookId: "{}") {{ eventId operation title }} }}"#,
        book_id
    );
    let (_, response) = graphql_request(&history_query, Some(&token)).await?;
    let entries = response["data"]["bookEvents"]
        .as_array()
        .context("bookEvents should be an array")?;
    let create_event_id = entries[0]["eventId"]
        .as_str()
        .context("eventId should be a string")?
        .to_owned();

    // Update the book
    let update_query = format!(
        r#"
        mutation {{
            updateBook(bookData: {{
                id: "{}"
                title: "After Update"
                authorIds: ["{}"]
                isbn: ""
                read: true
                owned: false
                priority: 50
                format: E_BOOK
                store: KINDLE
            }}) {{ book {{ id title }} eventSetId }}
        }}
        "#,
        book_id, author_id
    );
    graphql_request(&update_query, Some(&token)).await?;

    // Verify current title is "After Update"
    let book_query = format!(r#"{{ book(id: "{}") {{ title read }} }}"#, book_id);
    let (_, response) = graphql_request(&book_query, Some(&token)).await?;
    assert_eq!(
        response["data"]["book"]["title"].as_str(),
        Some("After Update"),
        "title should be 'After Update' before restore"
    );

    // Restore to the create event state
    let restore_query = format!(
        r#"mutation {{ restoreBook(eventId: "{}") {{ book {{ id title read }} eventSetId }} }}"#,
        create_event_id
    );
    let (_, response) = graphql_request(&restore_query, Some(&token)).await?;
    assert!(
        response.get("errors").is_none(),
        "restoreBook should not return errors: {:?}",
        response.get("errors")
    );
    assert_eq!(
        response["data"]["restoreBook"]["book"]["title"].as_str(),
        Some("Before Restore"),
        "restored book should have create-event title"
    );
    assert_eq!(
        response["data"]["restoreBook"]["book"]["read"].as_bool(),
        Some(false),
        "restored book should have create-event read flag"
    );

    // Verify that the book in the DB reflects the restored state
    let (_, response) = graphql_request(&book_query, Some(&token)).await?;
    assert_eq!(
        response["data"]["book"]["title"].as_str(),
        Some("Before Restore"),
        "book should reflect restored title"
    );

    // Cleanup
    delete_test_book(&book_id, &token).await?;
    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_restore_author_reverts_to_snapshot() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let original_name = format!("Restore Author Original {}", uuid::Uuid::new_v4());
    let author_id = create_test_author(&original_name, &token).await?;

    // Capture the create event's event_id
    let history_query = format!(
        r#"{{ authorEvents(authorId: "{}") {{ eventId operation name }} }}"#,
        author_id
    );
    let (_, response) = graphql_request(&history_query, Some(&token)).await?;
    let entries = response["data"]["authorEvents"]
        .as_array()
        .context("authorEvents should be an array")?;
    let create_event_id = entries[0]["eventId"]
        .as_str()
        .context("eventId should be a string")?
        .to_owned();

    // Update the author
    let updated_name = format!("Restore Author Updated {}", uuid::Uuid::new_v4());
    let update_query = format!(
        r#"mutation {{ updateAuthor(authorData: {{ id: "{}", name: "{}" }}) {{ author {{ id name }} eventSetId }} }}"#,
        author_id, updated_name
    );
    let (_, update_response) = graphql_request(&update_query, Some(&token)).await?;
    assert!(
        update_response.get("errors").is_none(),
        "updateAuthor should not return errors"
    );
    assert_eq!(
        update_response["data"]["updateAuthor"]["author"]["name"].as_str(),
        Some(updated_name.as_str()),
        "updateAuthor should return updated name"
    );

    // Restore to the create event state
    let restore_query = format!(
        r#"mutation {{ restoreAuthor(eventId: "{}") {{ author {{ id name }} eventSetId }} }}"#,
        create_event_id
    );
    let (_, response) = graphql_request(&restore_query, Some(&token)).await?;
    assert!(
        response.get("errors").is_none(),
        "restoreAuthor should not return errors: {:?}",
        response.get("errors")
    );
    assert_eq!(
        response["data"]["restoreAuthor"]["author"]["name"].as_str(),
        Some(original_name.as_str()),
        "restored author should have create-event name"
    );

    // Verify DB reflects the restored state
    let author_query = format!(r#"{{ author(id: "{}") {{ name }} }}"#, author_id);
    let (_, response) = graphql_request(&author_query, Some(&token)).await?;
    assert_eq!(
        response["data"]["author"]["name"].as_str(),
        Some(original_name.as_str()),
        "author in DB should reflect restored name"
    );

    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_restore_book_records_restore_event() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let author_id = create_test_author(
        &format!("Restore Event Author {}", uuid::Uuid::new_v4()),
        &token,
    )
    .await?;
    let book_id = create_test_book("Original Title", &author_id, &token).await?;

    // Get the create event_id
    let history_query = format!(
        r#"{{ bookEvents(bookId: "{}") {{ eventId operation extra }} }}"#,
        book_id
    );
    let (_, response) = graphql_request(&history_query, Some(&token)).await?;
    let entries = response["data"]["bookEvents"]
        .as_array()
        .context("bookEvents should be an array")?;
    let create_event_id = entries[0]["eventId"]
        .as_str()
        .context("eventId should be a string")?
        .to_owned();

    // Restore to the create event
    let restore_query = format!(
        r#"mutation {{ restoreBook(eventId: "{}") {{ book {{ id title }} eventSetId }} }}"#,
        create_event_id
    );
    graphql_request(&restore_query, Some(&token)).await?;

    // History should now include a restore event as the most recent entry
    let (_, response) = graphql_request(&history_query, Some(&token)).await?;
    let entries = response["data"]["bookEvents"]
        .as_array()
        .context("bookEvents should be an array after restore")?;

    assert!(
        entries.len() >= 2,
        "should have at least 2 entries after restore"
    );
    assert_eq!(
        entries[0]["operation"].as_str(),
        Some("restore"),
        "most recent entry should be 'restore'"
    );
    // extra should contain source_event_id
    let extra = &entries[0]["extra"];
    assert!(!extra.is_null(), "restore event extra should not be null");
    let create_event_id_i64: i64 = create_event_id.parse().context("event_id should be i64")?;
    assert_eq!(
        extra["source_event_id"].as_i64(),
        Some(create_event_id_i64),
        "restore event extra should contain the source event id"
    );
    assert_eq!(
        extra["version"].as_i64(),
        Some(1),
        "restore event extra should contain version 1"
    );

    delete_test_book(&book_id, &token).await?;
    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_restore_author_records_restore_event() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let author_name = format!("Restore Event Author {}", uuid::Uuid::new_v4());
    let author_id = create_test_author(&author_name, &token).await?;

    // Get the create event_id
    let history_query = format!(
        r#"{{ authorEvents(authorId: "{}") {{ eventId operation extra }} }}"#,
        author_id
    );
    let (_, response) = graphql_request(&history_query, Some(&token)).await?;
    let entries = response["data"]["authorEvents"]
        .as_array()
        .context("authorEvents should be an array")?;
    let create_event_id = entries[0]["eventId"]
        .as_str()
        .context("eventId should be a string")?
        .to_owned();

    // Restore to the create event
    let restore_query = format!(
        r#"mutation {{ restoreAuthor(eventId: "{}") {{ author {{ id name }} eventSetId }} }}"#,
        create_event_id
    );
    graphql_request(&restore_query, Some(&token)).await?;

    // History should now include a restore event as the most recent entry
    let (_, response) = graphql_request(&history_query, Some(&token)).await?;
    let entries = response["data"]["authorEvents"]
        .as_array()
        .context("authorEvents should be an array after restore")?;

    assert!(
        entries.len() >= 2,
        "should have at least 2 entries after restore"
    );
    assert_eq!(
        entries[0]["operation"].as_str(),
        Some("restore"),
        "most recent entry should be 'restore'"
    );
    let extra = &entries[0]["extra"];
    assert!(!extra.is_null(), "restore event extra should not be null");
    let create_event_id_i64: i64 = create_event_id.parse().context("event_id should be i64")?;
    assert_eq!(
        extra["source_event_id"].as_i64(),
        Some(create_event_id_i64),
        "restore event extra should contain the source event id"
    );
    assert_eq!(
        extra["version"].as_i64(),
        Some(1),
        "restore event extra should contain version 1"
    );

    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_restore_mutations_reject_invalid_or_missing_event_ids() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let invalid_queries = [
        r#"mutation { restoreBook(eventId: "not-an-int") { book { id } } }"#,
        r#"mutation { restoreAuthor(eventId: "not-an-int") { author { id } } }"#,
        r#"mutation { restoreBook(eventId: "999999999") { book { id } } }"#,
        r#"mutation { restoreAuthor(eventId: "999999999") { author { id } } }"#,
    ];

    for query in invalid_queries {
        let (_, response) = graphql_request(query, Some(&token)).await?;
        assert_graphql_errors(&response, "restore with an invalid or missing event id");
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_restore_mutations_are_user_isolated() -> Result<()> {
    let (_owner_user_id, owner_token) = create_test_user().await?;
    let (_other_user_id, other_token) = create_test_user().await?;

    let author_name = format!("Restore Isolation Author {}", uuid::Uuid::new_v4());
    let author_id = create_test_author(&author_name, &owner_token).await?;
    let book_id = create_test_book("Restore Isolation Book", &author_id, &owner_token).await?;

    let book_events_query = format!(
        r#"{{ bookEvents(bookId: "{}") {{ eventId operation }} }}"#,
        book_id
    );
    let (_, response) = graphql_request(&book_events_query, Some(&owner_token)).await?;
    let book_event_id = response["data"]["bookEvents"][0]["eventId"]
        .as_str()
        .context("book event id should be a string")?
        .to_owned();

    let author_events_query = format!(
        r#"{{ authorEvents(authorId: "{}") {{ eventId operation }} }}"#,
        author_id
    );
    let (_, response) = graphql_request(&author_events_query, Some(&owner_token)).await?;
    let author_event_id = response["data"]["authorEvents"][0]["eventId"]
        .as_str()
        .context("author event id should be a string")?
        .to_owned();

    let restore_book_query = format!(
        r#"mutation {{ restoreBook(eventId: "{}") {{ book {{ id }} eventSetId }} }}"#,
        book_event_id
    );
    let (_, response) = graphql_request(&restore_book_query, Some(&other_token)).await?;
    assert_graphql_errors(&response, "other user's restoreBook");

    let restore_author_query = format!(
        r#"mutation {{ restoreAuthor(eventId: "{}") {{ author {{ id }} eventSetId }} }}"#,
        author_event_id
    );
    let (_, response) = graphql_request(&restore_author_query, Some(&other_token)).await?;
    assert_graphql_errors(&response, "other user's restoreAuthor");

    let owner_book_query = format!(r#"{{ book(id: "{}") {{ title }} }}"#, book_id);
    let (_, response) = graphql_request(&owner_book_query, Some(&owner_token)).await?;
    assert_eq!(
        response["data"]["book"]["title"].as_str(),
        Some("Restore Isolation Book"),
        "other user's restoreBook should not alter owner data"
    );
    let owner_author_query = format!(r#"{{ author(id: "{}") {{ name }} }}"#, author_id);
    let (_, response) = graphql_request(&owner_author_query, Some(&owner_token)).await?;
    assert_eq!(
        response["data"]["author"]["name"].as_str(),
        Some(author_name.as_str()),
        "other user's restoreAuthor should not alter owner data"
    );

    delete_test_book(&book_id, &owner_token).await?;
    delete_test_author(&author_id, &owner_token).await?;
    Ok(())
}
