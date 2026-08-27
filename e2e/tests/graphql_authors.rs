// E2E tests that run against a real Postgres instance.

#![cfg(test)]

use anyhow::{Context, Result};
use bookshelf_e2e::*;
use serial_test::serial;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

fn parse_timestamp(value: &serde_json::Value, field: &str) -> Result<OffsetDateTime> {
    let value = value
        .as_str()
        .with_context(|| format!("{field} should be a string"))?;
    OffsetDateTime::parse(value, &Rfc3339)
        .with_context(|| format!("{field} should be an RFC 3339 timestamp"))
}

fn normalize_timestamp_for_comparison(value: OffsetDateTime) -> OffsetDateTime {
    let nanosecond = value.nanosecond() / 1_000 * 1_000;
    value
        .replace_nanosecond(nanosecond)
        .expect("truncated nanoseconds are always valid")
}

#[tokio::test]
#[serial]
async fn e2e_graphql_authors() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    // The authors list is user-scoped, so a fresh user starts empty.
    let query = r#"{ authors { id name yomi } }"#;
    let (_, response) = graphql_request(query, Some(&token)).await?;
    let authors = response["data"]["authors"]
        .as_array()
        .context("authors should be an array")?;
    assert!(authors.is_empty(), "a fresh user should have no authors");

    // After creating one author, it should appear in the list.
    let author_name = format!("Listed Author {}", uuid::Uuid::new_v4());
    let author_id = create_test_author(&author_name, &token).await?;

    let (_, response) = graphql_request(query, Some(&token)).await?;
    let authors = response["data"]["authors"]
        .as_array()
        .context("authors should be an array")?;
    assert_eq!(authors.len(), 1, "should list exactly the created author");
    assert_eq!(authors[0]["id"].as_str(), Some(author_id.as_str()));
    assert_eq!(authors[0]["name"].as_str(), Some(author_name.as_str()));
    assert_eq!(authors[0]["yomi"].as_str(), Some(""));

    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_authors_resolve_shared_and_empty_books() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;
    let author1_id = create_test_author("Author With Shared Book 1", &token).await?;
    let author2_id = create_test_author("Author With Shared Book 2", &token).await?;
    let empty_author_id = create_test_author("Author Without Books", &token).await?;
    let book_title = format!("Shared Book {}", uuid::Uuid::new_v4());
    let create_book = format!(
        r#"
        mutation {{
            createBook(bookData: {{
                title: "{}"
                authorIds: ["{}", "{}"]
                isbn: ""
                read: false
                owned: false
                priority: 50
                format: E_BOOK
                store: KINDLE
            }}) {{ book {{ id }} }}
        }}
        "#,
        book_title, author1_id, author2_id
    );
    let (_, response) = graphql_request(&create_book, Some(&token)).await?;
    assert_no_graphql_errors(&response, "create shared book");
    let book_id = response["data"]["createBook"]["book"]["id"]
        .as_str()
        .context("created book id should be a string")?
        .to_owned();

    let query = r#"{ authors { id books { id title } } }"#;
    let (_, response) = graphql_request(query, Some(&token)).await?;
    assert_no_graphql_errors(&response, "resolve author books");
    let authors = response["data"]["authors"]
        .as_array()
        .context("authors should be an array")?;
    let books_for = |author_id: &str| -> Result<&Vec<serde_json::Value>> {
        authors
            .iter()
            .find(|author| author["id"].as_str() == Some(author_id))
            .with_context(|| format!("author {author_id} should be returned"))?["books"]
            .as_array()
            .context("books should be an array")
    };

    for author_id in [&author1_id, &author2_id] {
        let books = books_for(author_id)?;
        assert_eq!(books.len(), 1);
        assert_eq!(books[0]["id"].as_str(), Some(book_id.as_str()));
        assert_eq!(books[0]["title"].as_str(), Some(book_title.as_str()));
    }
    assert!(
        books_for(&empty_author_id)?.is_empty(),
        "an author without books should return an empty list"
    );

    delete_test_book(&book_id, &token).await?;
    for author_id in [&author1_id, &author2_id, &empty_author_id] {
        delete_test_author(author_id, &token).await?;
    }
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_merge_author_moves_books_and_returns_destination() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;
    let source_id = create_test_author("Merge Source", &token).await?;
    let destination_id = create_test_author("Merge Destination", &token).await?;
    let book1_id = create_test_book("Merged Book 1", &source_id, &token).await?;
    let book2_id = create_test_book("Merged Book 2", &source_id, &token).await?;
    let create_shared_book = format!(
        r#"mutation {{
            createBook(bookData: {{
                title: "Merged Book With Destination"
                authorIds: ["{}", "{}"]
                isbn: ""
                read: false
                owned: false
                priority: 50
                format: E_BOOK
                store: KINDLE
            }}) {{ book {{ id }} }}
        }}"#,
        source_id, destination_id
    );
    let (_, response) = graphql_request(&create_shared_book, Some(&token)).await?;
    assert_no_graphql_errors(&response, "create merge book with destination");
    let shared_book_id = response["data"]["createBook"]["book"]["id"]
        .as_str()
        .context("shared merge book id should be a string")?
        .to_owned();

    let mutation = format!(
        r#"mutation {{ mergeAuthor(sourceAuthorId: "{}", destinationAuthorId: "{}") {{ author {{ id name books {{ id }} }} operationId }} }}"#,
        source_id, destination_id
    );
    let (_, response) = graphql_request(&mutation, Some(&token)).await?;
    assert_no_graphql_errors(&response, "merge author");
    let payload = &response["data"]["mergeAuthor"];
    assert_eq!(
        payload["author"]["id"].as_str(),
        Some(destination_id.as_str())
    );
    let merged_book_ids: std::collections::HashSet<&str> = payload["author"]["books"]
        .as_array()
        .context("merged destination books should be an array")?
        .iter()
        .filter_map(|book| book["id"].as_str())
        .collect();
    assert_eq!(
        merged_book_ids,
        [&*book1_id, &*book2_id, &*shared_book_id]
            .into_iter()
            .collect()
    );
    let operation_id = payload["operationId"]
        .as_str()
        .context("merge should return operationId")?;
    assert!(payload.get("revisionNumber").is_none());

    let query = format!(
        r#"{{ source: author(id: "{}") {{ id }} destination: author(id: "{}") {{ id books {{ id }} }} }}"#,
        source_id, destination_id
    );
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    assert!(response["data"]["source"].is_null());
    assert_eq!(
        response["data"]["destination"]["books"]
            .as_array()
            .unwrap()
            .len(),
        3
    );

    let operation_query = format!(
        r#"{{ operation(id: "{}") {{
            type detail
            bookChanges {{ bookId afterRevision {{ authorIds }} }}
            authorChanges {{ authorId beforeRevision {{ revisionNumber }} afterRevision {{ revisionNumber }} }}
        }} }}"#,
        operation_id
    );
    let (_, response) = graphql_request(&operation_query, Some(&token)).await?;
    assert_no_graphql_errors(&response, "merge operation");
    let operation = &response["data"]["operation"];
    assert_eq!(operation["type"].as_str(), Some("merge_author"));
    let book_events = operation["bookChanges"]
        .as_array()
        .context("merge book events should be an array")?;
    assert_eq!(book_events.len(), 3);
    for event in book_events {
        assert_eq!(
            event["afterRevision"]["authorIds"].as_array().unwrap(),
            &[serde_json::Value::String(destination_id.clone())]
        );
    }
    let author_events = operation["authorChanges"]
        .as_array()
        .context("merge author events should be an array")?;
    assert_eq!(author_events.len(), 2);
    assert!(author_events.iter().any(|event| {
        event["authorId"].as_str() == Some(source_id.as_str()) && event["afterRevision"].is_null()
    }));
    assert!(author_events.iter().any(|event| {
        event["authorId"].as_str() == Some(destination_id.as_str())
            && !event["afterRevision"].is_null()
    }));

    for book_id in [&book1_id, &book2_id, &shared_book_id] {
        delete_test_book(book_id, &token).await?;
    }
    delete_test_author(&destination_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_create_author() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    // Use a random name to keep the author unique across runs.
    let random_name = format!("Test Author {}", uuid::Uuid::new_v4());
    let yomi = "てすと・おーさー1";

    let query = format!(
        r#"mutation {{ createAuthor(authorData: {{ name: "{}", yomi: "{}" }}) {{ author {{ id name yomi createdAt updatedAt }} operationId }} }}"#,
        random_name, yomi
    );
    let before = normalize_timestamp_for_comparison(OffsetDateTime::now_utc());
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    let after = normalize_timestamp_for_comparison(OffsetDateTime::now_utc());

    let data = response.get("data").context("data field must exist")?;
    let create_result = data
        .get("createAuthor")
        .context("createAuthor field must exist")?
        .get("author")
        .context("author field must exist")?;
    let author_id = create_result
        .get("id")
        .context("id field must exist")?
        .as_str()
        .context("id should be string")?;
    assert_eq!(
        create_result
            .get("name")
            .context("name field must exist")?
            .as_str(),
        Some(random_name.as_str())
    );
    assert_eq!(create_result["yomi"].as_str(), Some(yomi));
    let created_at = parse_timestamp(&create_result["createdAt"], "createdAt")?;
    let updated_at = parse_timestamp(&create_result["updatedAt"], "updatedAt")?;
    assert_eq!(created_at, updated_at);
    assert!(created_at >= before);
    assert!(created_at <= after);

    // Verify author was created by fetching it
    let author_query = format!(
        r#"{{ author(id: "{}") {{ id name yomi createdAt updatedAt }} }}"#,
        author_id
    );
    let (_, response) = graphql_request(&author_query, Some(&token)).await?;
    let data = response.get("data").context("data field must exist")?;
    let author = data.get("author").context("author field must exist")?;
    assert!(!author.is_null(), "author should exist after creation");
    let author_name_from_query = author
        .get("name")
        .context("name field must exist")?
        .as_str()
        .context("name should be string")?;
    assert_eq!(
        author_name_from_query, random_name,
        "author name should match"
    );
    assert_eq!(author["yomi"].as_str(), Some(yomi));
    assert_eq!(
        parse_timestamp(&author["createdAt"], "createdAt")?,
        created_at
    );
    assert_eq!(
        parse_timestamp(&author["updatedAt"], "updatedAt")?,
        updated_at
    );

    delete_test_author(author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_delete_author_without_books_succeeds() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let random_name = format!("Author To Delete {}", uuid::Uuid::new_v4());
    let author_id = create_test_author(&random_name, &token).await?;

    delete_test_author(&author_id, &token).await?;

    // Verify author no longer exists
    let query = format!(r#"{{ author(id: "{}") {{ id }} }}"#, author_id);
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    assert!(
        response["data"]["author"].is_null(),
        "author should be null after deletion"
    );
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_delete_author_with_associated_books_fails() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    // Create author
    let random_name = format!("Author With Book {}", uuid::Uuid::new_v4());
    let author_id = create_test_author(&random_name, &token).await?;

    // Create book associated with the author
    let book_id = create_test_book("Book Blocking Author Delete", &author_id, &token).await?;

    // Attempt to delete the author — must fail
    let delete_author_query = format!(
        r#"mutation {{ deleteAuthor(authorId: "{}") {{ authorId }} }}"#,
        author_id
    );
    let (_, response) = graphql_request(&delete_author_query, Some(&token)).await?;
    assert!(
        response.get("errors").is_some(),
        "deleteAuthor should return errors when author has associated books"
    );

    // Verify the author still exists
    let query = format!(r#"{{ author(id: "{}") {{ id }} }}"#, author_id);
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    assert!(
        !response["data"]["author"].is_null(),
        "author should still exist after failed deletion"
    );

    // Clean up: delete book first, then author
    delete_test_book(&book_id, &token).await?;
    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_update_author() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    // Create author
    let original_name = format!("Author Before Update {}", uuid::Uuid::new_v4());
    let author_id = create_test_author(&original_name, &token).await?;

    let initial_query = format!(
        r#"{{ author(id: "{}") {{ createdAt updatedAt }} }}"#,
        author_id
    );
    let (_, response) = graphql_request(&initial_query, Some(&token)).await?;
    let initial_author = &response["data"]["author"];
    let created_at = parse_timestamp(&initial_author["createdAt"], "createdAt")?;
    let previous_updated_at = parse_timestamp(&initial_author["updatedAt"], "updatedAt")?;

    // Create book associated with the author
    let book_id = create_test_book("Book For Author Update Test", &author_id, &token).await?;

    // Update author name while the author has an associated book
    let updated_name = format!("Author After Update {}", uuid::Uuid::new_v4());
    let updated_yomi = "こうしんご2";
    let update_query = format!(
        r#"mutation {{ updateAuthor(authorData: {{ id: "{}", name: "{}", yomi: "{}" }}) {{ author {{ id name yomi createdAt updatedAt }} operationId }} }}"#,
        author_id, updated_name, updated_yomi
    );
    let before = normalize_timestamp_for_comparison(OffsetDateTime::now_utc());
    let (_, response) = graphql_request(&update_query, Some(&token)).await?;
    let after = normalize_timestamp_for_comparison(OffsetDateTime::now_utc());
    assert!(
        response.get("errors").is_none(),
        "updateAuthor should not return errors"
    );
    let update_result = &response["data"]["updateAuthor"]["author"];
    assert_eq!(
        update_result["id"].as_str(),
        Some(author_id.as_str()),
        "updated author id should match"
    );
    assert_eq!(
        update_result["name"].as_str(),
        Some(updated_name.as_str()),
        "updated author name should match"
    );
    assert_eq!(update_result["yomi"].as_str(), Some(updated_yomi));
    assert_eq!(
        parse_timestamp(&update_result["createdAt"], "createdAt")?,
        created_at
    );
    let updated_at = parse_timestamp(&update_result["updatedAt"], "updatedAt")?;
    assert!(updated_at >= previous_updated_at);
    assert!(updated_at >= before);
    assert!(updated_at <= after);

    // Omitting yomi preserves the current value.
    let preserved_name = format!("Author Preserving Yomi {}", uuid::Uuid::new_v4());
    let preserve_query = format!(
        r#"mutation {{ updateAuthor(authorData: {{ id: "{}", name: "{}" }}) {{ author {{ yomi }} }} }}"#,
        author_id, preserved_name
    );
    let (_, response) = graphql_request(&preserve_query, Some(&token)).await?;
    assert_eq!(
        response["data"]["updateAuthor"]["author"]["yomi"].as_str(),
        Some(updated_yomi)
    );

    // An explicit empty string clears yomi.
    let clear_query = format!(
        r#"mutation {{ updateAuthor(authorData: {{ id: "{}", name: "{}", yomi: "" }}) {{ author {{ yomi }} }} }}"#,
        author_id, preserved_name
    );
    let (_, response) = graphql_request(&clear_query, Some(&token)).await?;
    assert_eq!(
        response["data"]["updateAuthor"]["author"]["yomi"].as_str(),
        Some("")
    );

    // Verify update by fetching the author
    let query = format!(
        r#"{{ author(id: "{}") {{ id name createdAt updatedAt }} }}"#,
        author_id
    );
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    assert_eq!(
        response["data"]["author"]["name"].as_str(),
        Some(preserved_name.as_str()),
        "author name should reflect the update"
    );
    assert_eq!(
        parse_timestamp(&response["data"]["author"]["createdAt"], "createdAt")?,
        created_at
    );
    assert!(parse_timestamp(&response["data"]["author"]["updatedAt"], "updatedAt")? >= updated_at);

    // Clean up: delete book first, then author
    delete_test_book(&book_id, &token).await?;
    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_update_nonexistent_author_returns_error() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let nonexistent_id = uuid::Uuid::new_v4().to_string();
    let query = format!(
        r#"mutation {{ updateAuthor(authorData: {{ id: "{}", name: "Ghost" }}) {{ author {{ id name }} operationId }} }}"#,
        nonexistent_id
    );
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    assert!(
        response.get("errors").is_some(),
        "updateAuthor should return errors for a non-existent author"
    );
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_delete_nonexistent_author_returns_error() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let nonexistent_id = uuid::Uuid::new_v4().to_string();
    let query = format!(
        r#"mutation {{ deleteAuthor(authorId: "{}") {{ authorId }} }}"#,
        nonexistent_id
    );
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    assert!(
        response.get("errors").is_some(),
        "deleteAuthor should return errors for a non-existent author"
    );
    Ok(())
}
