// E2E tests that run against a real Postgres instance.

#![cfg(test)]

use anyhow::{Context, Result};
use bookshelf_e2e::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn e2e_event_sets() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    // Two separate operations => two event sets (create_author, then create_book).
    let author_id = create_test_author(
        &format!("Event Set Author {}", uuid::Uuid::new_v4()),
        &token,
    )
    .await?;
    let book_id = create_test_book("Event Set Book", &author_id, &token).await?;

    // List event sets: newest first, exactly the two operations above.
    let (_, response) =
        graphql_request(r#"{ eventSets { id operation createdAt } }"#, Some(&token)).await?;
    let sets = response["data"]["eventSets"]
        .as_array()
        .context("eventSets should be an array")?;
    assert_eq!(sets.len(), 2, "fresh user should have exactly 2 event sets");
    // Ordered by createdAt descending (newest first).
    let created0 = sets[0]["createdAt"].as_i64().context("createdAt is i64")?;
    let created1 = sets[1]["createdAt"].as_i64().context("createdAt is i64")?;
    assert!(created0 >= created1, "event sets should be newest first");
    let operations: Vec<&str> = sets
        .iter()
        .filter_map(|s| s["operation"].as_str())
        .collect();
    assert!(
        operations.contains(&"create_book"),
        "should contain a create_book event set"
    );
    assert!(
        operations.contains(&"create_author"),
        "should contain a create_author event set"
    );

    // Drill into the create_book event set: it groups the book's create event.
    let create_book_set_id = sets
        .iter()
        .find(|s| s["operation"].as_str() == Some("create_book"))
        .and_then(|s| s["id"].as_str())
        .context("create_book event set should have an id")?;
    let detail_query = format!(
        r#"{{ eventSet(id: "{}") {{
            id operation createdAt
            bookEvents {{ bookId operation title changedAt }}
            authorEvents {{ name operation }}
        }} }}"#,
        create_book_set_id
    );
    let (_, response) = graphql_request(&detail_query, Some(&token)).await?;
    let detail = &response["data"]["eventSet"];
    assert!(!detail.is_null(), "eventSet should be found");
    assert_eq!(
        detail["id"].as_str(),
        Some(create_book_set_id),
        "eventSet id should round-trip the queried id"
    );
    assert_eq!(detail["operation"].as_str(), Some("create_book"));
    assert!(
        detail["createdAt"].as_i64().is_some(),
        "createdAt should be an Int"
    );
    let book_events = detail["bookEvents"]
        .as_array()
        .context("bookEvents should be an array")?;
    assert_eq!(book_events.len(), 1, "create_book set has one book event");
    assert_eq!(book_events[0]["bookId"].as_str(), Some(book_id.as_str()));
    assert_eq!(book_events[0]["operation"].as_str(), Some("create"));
    // The nested entry must carry the full BookEventEntry data, not just ids
    // (guards the find_by_event_set projection).
    assert_eq!(
        book_events[0]["title"].as_str(),
        Some("Event Set Book"),
        "nested book event should carry the title"
    );
    assert!(
        !book_events[0]["changedAt"].is_null(),
        "nested book event changedAt should not be null"
    );
    assert!(
        detail["authorEvents"]
            .as_array()
            .context("authorEvents should be an array")?
            .is_empty(),
        "create_book set has no author events"
    );

    // Drill into the create_author event set: it groups the author's create event.
    let create_author_set_id = sets
        .iter()
        .find(|s| s["operation"].as_str() == Some("create_author"))
        .and_then(|s| s["id"].as_str())
        .context("create_author event set should have an id")?;
    let detail_query = format!(
        r#"{{ eventSet(id: "{}") {{ authorEvents {{ name operation }} bookEvents {{ bookId }} }} }}"#,
        create_author_set_id
    );
    let (_, response) = graphql_request(&detail_query, Some(&token)).await?;
    let author_events = response["data"]["eventSet"]["authorEvents"]
        .as_array()
        .context("authorEvents should be an array")?;
    assert_eq!(
        author_events.len(),
        1,
        "create_author set has one author event"
    );
    assert_eq!(author_events[0]["operation"].as_str(), Some("create"));

    // Unknown id returns null.
    let unknown_query = format!(r#"{{ eventSet(id: "{}") {{ id }} }}"#, uuid::Uuid::new_v4());
    let (_, response) = graphql_request(&unknown_query, Some(&token)).await?;
    assert!(
        response["data"]["eventSet"].is_null(),
        "unknown event set id should return null"
    );

    // User isolation: a second user cannot see the first user's event set.
    let (_other_user_id, other_token) = create_test_user().await?;
    let isolation_query = format!(r#"{{ eventSet(id: "{}") {{ id }} }}"#, create_book_set_id);
    let (_, response) = graphql_request(&isolation_query, Some(&other_token)).await?;
    assert!(
        response["data"]["eventSet"].is_null(),
        "other user must not see another user's event set"
    );
    let (_, response) = graphql_request(r#"{ eventSets { id } }"#, Some(&other_token)).await?;
    assert!(
        response["data"]["eventSets"]
            .as_array()
            .context("eventSets should be an array")?
            .is_empty(),
        "a fresh user should have no event sets"
    );

    // Cleanup.
    delete_test_book(&book_id, &token).await?;
    delete_test_author(&author_id, &token).await?;

    Ok(())
}
