// E2E tests that run against a real Postgres instance.

#![cfg(test)]

use anyhow::{Context, Result};
use bookshelf_e2e::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn e2e_import_books() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    // Pre-create an author
    let existing_author_id = create_test_author("Existing Author", &token).await?;

    // Call importBooks
    let import_query = r#"
        mutation {
            importBooks(books: [
                {
                    title: "Book One"
                    authorNames: ["Existing Author"]
                    isbn: ""
                    read: false
                    owned: false
                    priority: 50
                    format: E_BOOK
                    store: KINDLE
                },
                {
                    title: "Book Two"
                    authorNames: ["New Author"]
                    isbn: ""
                    read: false
                    owned: false
                    priority: 50
                    format: E_BOOK
                    store: KINDLE
                }
            ]) {
                id
                title
                authors {
                    name
                }
            }
        }
    "#;
    let (_, response) = graphql_request(import_query, Some(&token)).await?;
    assert!(
        response.get("errors").is_none(),
        "importBooks should not return errors: {:?}",
        response.get("errors")
    );

    let imported_books = response["data"]["importBooks"]
        .as_array()
        .context("importBooks should return an array")?;
    assert_eq!(imported_books.len(), 2, "should import exactly 2 books");

    let book_one_id = imported_books[0]["id"]
        .as_str()
        .context("book id should be string")?;
    let book_one_title = imported_books[0]["title"]
        .as_str()
        .context("book title should be string")?;
    let book_one_authors = imported_books[0]["authors"]
        .as_array()
        .context("book one authors should be an array")?;
    let book_two_id = imported_books[1]["id"]
        .as_str()
        .context("book id should be string")?;
    let book_two_title = imported_books[1]["title"]
        .as_str()
        .context("book title should be string")?;
    let book_two_authors = imported_books[1]["authors"]
        .as_array()
        .context("book two authors should be an array")?;

    assert_eq!(book_one_title, "Book One");
    assert_eq!(book_one_authors.len(), 1);
    assert_eq!(
        book_one_authors[0]["name"].as_str(),
        Some("Existing Author")
    );
    assert_eq!(book_two_title, "Book Two");
    assert_eq!(book_two_authors.len(), 1);
    assert_eq!(book_two_authors[0]["name"].as_str(), Some("New Author"));
    assert!(
        !book_one_id.is_empty(),
        "book one should have a non-empty id"
    );
    assert!(
        !book_two_id.is_empty(),
        "book two should have a non-empty id"
    );
    assert_ne!(book_one_id, book_two_id, "book ids should be distinct");

    // Locate the single event set produced by the import via eventSets, then
    // inspect it directly. This replaces the old workaround of comparing the
    // eventSetId across separate per-book bookEvents queries.
    let (_, response) = graphql_request(r#"{ eventSets { id operation } }"#, Some(&token)).await?;
    let event_sets = response["data"]["eventSets"]
        .as_array()
        .context("eventSets should be an array")?;
    let import_set_id = event_sets
        .iter()
        .find(|s| s["operation"].as_str() == Some("import_books"))
        .and_then(|s| s["id"].as_str())
        .context("there should be an import_books event set")?;

    // The shared event set groups both book creates and the new author create.
    let event_set_query = format!(
        r#"{{ eventSet(id: "{}") {{
            operation
            bookEvents {{ bookId operation }}
            authorEvents {{ name operation }}
        }} }}"#,
        import_set_id
    );
    let (_, response) = graphql_request(&event_set_query, Some(&token)).await?;
    let event_set = &response["data"]["eventSet"];
    assert!(!event_set.is_null(), "import event set should be found");
    assert_eq!(
        event_set["operation"].as_str(),
        Some("import_books"),
        "import event set operation should be import_books"
    );
    let grouped_book_events = event_set["bookEvents"]
        .as_array()
        .context("bookEvents should be an array")?;
    assert_eq!(
        grouped_book_events.len(),
        2,
        "import event set should group both book create events"
    );
    assert!(
        grouped_book_events
            .iter()
            .all(|e| e["operation"].as_str() == Some("create")),
        "all grouped book events should be create events"
    );
    let grouped_book_ids: Vec<&str> = grouped_book_events
        .iter()
        .filter_map(|e| e["bookId"].as_str())
        .collect();
    assert!(grouped_book_ids.contains(&book_one_id));
    assert!(grouped_book_ids.contains(&book_two_id));
    let grouped_author_events = event_set["authorEvents"]
        .as_array()
        .context("authorEvents should be an array")?;
    assert_eq!(
        grouped_author_events.len(),
        1,
        "import event set should group the newly created author event"
    );
    assert_eq!(
        grouped_author_events[0]["name"].as_str(),
        Some("New Author"),
        "grouped author event should be the new author"
    );
    assert_eq!(
        grouped_author_events[0]["operation"].as_str(),
        Some("create")
    );

    // Verify Book Two has "New Author"
    let book_two_query = format!(
        r#"{{ book(id: "{}") {{ authors {{ name }} }} }}"#,
        book_two_id
    );
    let (_, response) = graphql_request(&book_two_query, Some(&token)).await?;
    let authors = response["data"]["book"]["authors"]
        .as_array()
        .context("authors should be an array")?;
    assert_eq!(authors.len(), 1, "book two should have 1 author");
    assert_eq!(
        authors[0]["name"].as_str(),
        Some("New Author"),
        "book two author should be New Author"
    );

    // Verify "Existing Author" was not duplicated
    let authors_query = r#"{ authors { id name } }"#;
    let (_, response) = graphql_request(authors_query, Some(&token)).await?;
    let all_authors = response["data"]["authors"]
        .as_array()
        .context("authors should be an array")?;
    let existing_count = all_authors
        .iter()
        .filter(|a| a["name"].as_str() == Some("Existing Author"))
        .count();
    assert_eq!(
        existing_count, 1,
        "Existing Author should appear exactly once"
    );

    // Verify New Author has a create event in authorEvents
    let new_author = all_authors
        .iter()
        .find(|a| a["name"].as_str() == Some("New Author"));
    let new_author_id = new_author
        .and_then(|a| a["id"].as_str())
        .context("new author should have an id")?;
    let author_events_query = format!(
        r#"{{ authorEvents(authorId: "{}") {{ operation name }} }}"#,
        new_author_id
    );
    let (_, response) = graphql_request(&author_events_query, Some(&token)).await?;
    let author_events = response["data"]["authorEvents"]
        .as_array()
        .context("authorEvents should be an array")?;
    assert_eq!(
        author_events.len(),
        1,
        "new author should have exactly 1 event entry"
    );
    assert_eq!(
        author_events[0]["operation"].as_str(),
        Some("create"),
        "new author event should be create"
    );
    assert_eq!(
        author_events[0]["name"].as_str(),
        Some("New Author"),
        "new author event name should match"
    );

    // Cleanup
    delete_test_book(book_one_id, &token).await?;
    delete_test_book(book_two_id, &token).await?;
    delete_test_author(&existing_author_id, &token).await?;

    // Delete New Author
    delete_test_author(new_author_id, &token).await?;

    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_import_books_many_entries() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;
    let run_id = uuid::Uuid::new_v4();

    let existing_author_names: Vec<String> = (0..3)
        .map(|i| format!("Bulk Existing Author {run_id} {i}"))
        .collect();
    let mut author_ids = Vec::new();
    for name in &existing_author_names {
        author_ids.push(create_test_author(name, &token).await?);
    }

    let import_count = 25;
    let imported_entries = (0..import_count)
        .map(|i| {
            let title = format!("Bulk Import Book {run_id} {i:02}");
            let existing_author_name = &existing_author_names[i % existing_author_names.len()];
            let new_author_name = format!("Bulk New Author {run_id} {i:02}");
            format!(
                r#"{{
                    title: "{title}"
                    authorNames: ["{existing_author_name}", "{new_author_name}"]
                    isbn: "978000000{i:04}"
                    read: false
                    owned: true
                    priority: 50
                    format: E_BOOK
                    store: KINDLE
                }}"#
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");

    let import_query = format!(
        r#"
        mutation {{
            importBooks(books: [{imported_entries}]) {{
                id
                title
                authors {{ name }}
            }}
        }}
        "#
    );
    let (_, response) = graphql_request(&import_query, Some(&token)).await?;
    assert!(
        response.get("errors").is_none(),
        "bulk importBooks should not return errors: {:?}",
        response.get("errors")
    );

    let imported_books = response["data"]["importBooks"]
        .as_array()
        .context("importBooks should return an array")?;
    assert_eq!(
        imported_books.len(),
        import_count,
        "bulk import should return every requested book"
    );

    let mut imported_book_ids = Vec::new();
    for (i, book) in imported_books.iter().enumerate() {
        let book_id = book["id"]
            .as_str()
            .context("imported book id should be a string")?;
        imported_book_ids.push(book_id.to_owned());
        assert_eq!(
            book["title"].as_str(),
            Some(format!("Bulk Import Book {run_id} {i:02}").as_str())
        );
        let authors = book["authors"]
            .as_array()
            .context("imported book authors should be an array")?;
        assert_eq!(authors.len(), 2, "each bulk imported book has 2 authors");
        assert!(authors.iter().any(|author| {
            author["name"].as_str()
                == Some(existing_author_names[i % existing_author_names.len()].as_str())
        }));
        assert!(authors.iter().any(|author| {
            author["name"].as_str() == Some(format!("Bulk New Author {run_id} {i:02}").as_str())
        }));
    }

    let (_, response) = graphql_request(r#"{ eventSets { id operation } }"#, Some(&token)).await?;
    let event_sets = response["data"]["eventSets"]
        .as_array()
        .context("eventSets should be an array")?;
    let import_set_id = event_sets
        .iter()
        .find(|s| s["operation"].as_str() == Some("import_books"))
        .and_then(|s| s["id"].as_str())
        .context("there should be an import_books event set")?;

    let event_set_query = format!(
        r#"{{ eventSet(id: "{}") {{
            operation
            bookEvents {{ bookId operation }}
            authorEvents {{ name operation }}
        }} }}"#,
        import_set_id
    );
    let (_, response) = graphql_request(&event_set_query, Some(&token)).await?;
    let event_set = &response["data"]["eventSet"];
    assert_eq!(event_set["operation"].as_str(), Some("import_books"));
    let grouped_book_events = event_set["bookEvents"]
        .as_array()
        .context("bookEvents should be an array")?;
    assert_eq!(
        grouped_book_events.len(),
        import_count,
        "bulk import event set should group every book create event"
    );
    assert!(
        grouped_book_events
            .iter()
            .all(|event| event["operation"].as_str() == Some("create")),
        "all bulk import book events should be create events"
    );
    for book_id in &imported_book_ids {
        assert!(
            grouped_book_events
                .iter()
                .any(|event| event["bookId"].as_str() == Some(book_id.as_str())),
            "bulk import event set should contain book id {book_id}"
        );
    }

    let grouped_author_events = event_set["authorEvents"]
        .as_array()
        .context("authorEvents should be an array")?;
    assert_eq!(
        grouped_author_events.len(),
        import_count,
        "bulk import event set should only group newly created author events"
    );
    for i in 0..import_count {
        let new_author_name = format!("Bulk New Author {run_id} {i:02}");
        assert!(
            grouped_author_events.iter().any(|event| {
                event["operation"].as_str() == Some("create")
                    && event["name"].as_str() == Some(new_author_name.as_str())
            }),
            "bulk import event set should contain author event for {new_author_name}"
        );
    }

    let (_, response) = graphql_request(r#"{ authors { id name } }"#, Some(&token)).await?;
    let all_authors = response["data"]["authors"]
        .as_array()
        .context("authors should be an array")?;
    for existing_author_name in &existing_author_names {
        assert_eq!(
            all_authors
                .iter()
                .filter(|author| author["name"].as_str() == Some(existing_author_name.as_str()))
                .count(),
            1,
            "existing bulk import authors should not be duplicated"
        );
    }
    for i in 0..import_count {
        let new_author_name = format!("Bulk New Author {run_id} {i:02}");
        let new_author_id = all_authors
            .iter()
            .find(|author| author["name"].as_str() == Some(new_author_name.as_str()))
            .and_then(|author| author["id"].as_str())
            .context("new bulk import author should exist")?;
        author_ids.push(new_author_id.to_owned());
    }

    for book_id in &imported_book_ids {
        delete_test_book(book_id, &token).await?;
    }
    for author_id in &author_ids {
        delete_test_author(author_id, &token).await?;
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_import_books_rolls_back_when_one_entry_is_invalid() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;
    let run_id = uuid::Uuid::new_v4();
    let valid_title = format!("Rollback Import Valid Book {run_id}");
    let valid_author = format!("Rollback Import Valid Author {run_id}");
    let invalid_author = format!("Rollback Import Invalid Author {run_id}");

    let import_query = format!(
        r#"
        mutation {{
            importBooks(books: [
                {{
                    title: "{valid_title}"
                    authorNames: ["{valid_author}"]
                    isbn: ""
                    read: false
                    owned: false
                    priority: 50
                    format: E_BOOK
                    store: KINDLE
                }},
                {{
                    title: ""
                    authorNames: ["{invalid_author}"]
                    isbn: ""
                    read: false
                    owned: false
                    priority: 50
                    format: E_BOOK
                    store: KINDLE
                }}
            ]) {{ id }}
        }}
        "#
    );
    let (_, response) = graphql_request(&import_query, Some(&token)).await?;
    assert_graphql_errors(&response, "importBooks with one invalid entry");

    let (_, response) = graphql_request(
        r#"{ books { title } authors { name } eventSets { operation } }"#,
        Some(&token),
    )
    .await?;
    let books = response["data"]["books"]
        .as_array()
        .context("books should be an array")?;
    let authors = response["data"]["authors"]
        .as_array()
        .context("authors should be an array")?;
    let event_sets = response["data"]["eventSets"]
        .as_array()
        .context("eventSets should be an array")?;
    assert!(
        books
            .iter()
            .all(|book| book["title"].as_str() != Some(valid_title.as_str())),
        "failed import should not persist the valid book"
    );
    assert!(
        authors.iter().all(|author| {
            author["name"].as_str() != Some(valid_author.as_str())
                && author["name"].as_str() != Some(invalid_author.as_str())
        }),
        "failed import should not persist any import authors"
    );
    assert!(
        event_sets
            .iter()
            .all(|set| set["operation"].as_str() != Some("import_books")),
        "failed import should not create an import_books event set"
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_import_books_rejects_more_than_max_batch() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;
    let run_id = uuid::Uuid::new_v4();
    let imported_entries = (0..1001)
        .map(|i| {
            format!(
                r#"{{
                    title: "Too Many Import Book {run_id} {i:04}"
                    authorNames: []
                    isbn: ""
                    read: false
                    owned: false
                    priority: 50
                    format: E_BOOK
                    store: KINDLE
                }}"#
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    let import_query =
        format!(r#"mutation {{ importBooks(books: [{imported_entries}]) {{ id }} }}"#);

    let (_, response) = graphql_request(&import_query, Some(&token)).await?;
    assert_graphql_errors(&response, "importBooks above the max batch size");

    let (_, response) =
        graphql_request(r#"{ books { id } eventSets { operation } }"#, Some(&token)).await?;
    assert!(
        response["data"]["books"]
            .as_array()
            .context("books should be an array")?
            .is_empty(),
        "rejected oversized import should not create books"
    );
    assert!(
        response["data"]["eventSets"]
            .as_array()
            .context("eventSets should be an array")?
            .iter()
            .all(|set| set["operation"].as_str() != Some("import_books")),
        "rejected oversized import should not create an import event set"
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_import_books_deduplicates_shared_new_author() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;
    let run_id = uuid::Uuid::new_v4();
    let shared_author = format!("Shared Imported Author {run_id}");
    let titles: Vec<String> = (0..3)
        .map(|i| format!("Shared Author Import Book {run_id} {i}"))
        .collect();
    let imported_entries = titles
        .iter()
        .map(|title| {
            format!(
                r#"{{
                    title: "{title}"
                    authorNames: ["{shared_author}"]
                    isbn: ""
                    read: false
                    owned: false
                    priority: 50
                    format: E_BOOK
                    store: KINDLE
                }}"#
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    let import_query = format!(
        r#"
        mutation {{
            importBooks(books: [{imported_entries}]) {{
                id
                title
                authors {{ name }}
            }}
        }}
        "#
    );
    let (_, response) = graphql_request(&import_query, Some(&token)).await?;
    assert_no_graphql_errors(&response, "importBooks with a shared new author");

    let imported_books = response["data"]["importBooks"]
        .as_array()
        .context("importBooks should return an array")?;
    assert_eq!(imported_books.len(), titles.len());
    let imported_book_ids: Vec<String> = imported_books
        .iter()
        .map(|book| {
            let authors = book["authors"]
                .as_array()
                .context("book authors should be an array")?;
            assert_eq!(authors.len(), 1, "each book should have the shared author");
            assert_eq!(authors[0]["name"].as_str(), Some(shared_author.as_str()));
            book["id"]
                .as_str()
                .context("book id should be a string")
                .map(str::to_owned)
        })
        .collect::<Result<Vec<_>>>()?;

    let (_, response) = graphql_request(
        r#"{ authors { id name } eventSets { id operation } }"#,
        Some(&token),
    )
    .await?;
    let authors = response["data"]["authors"]
        .as_array()
        .context("authors should be an array")?;
    let matching_authors: Vec<&serde_json::Value> = authors
        .iter()
        .filter(|author| author["name"].as_str() == Some(shared_author.as_str()))
        .collect();
    assert_eq!(
        matching_authors.len(),
        1,
        "shared new author should be created once"
    );
    let shared_author_id = matching_authors[0]["id"]
        .as_str()
        .context("shared author id should be a string")?;

    let import_set_id = response["data"]["eventSets"]
        .as_array()
        .context("eventSets should be an array")?
        .iter()
        .find(|set| set["operation"].as_str() == Some("import_books"))
        .and_then(|set| set["id"].as_str())
        .context("there should be an import_books event set")?;
    let event_set_query = format!(
        r#"{{ eventSet(id: "{}") {{ bookEvents {{ bookId operation }} authorEvents {{ name operation }} }} }}"#,
        import_set_id
    );
    let (_, response) = graphql_request(&event_set_query, Some(&token)).await?;
    assert_eq!(
        response["data"]["eventSet"]["bookEvents"]
            .as_array()
            .context("bookEvents should be an array")?
            .len(),
        titles.len(),
        "each imported book should have a create event"
    );
    let author_events = response["data"]["eventSet"]["authorEvents"]
        .as_array()
        .context("authorEvents should be an array")?;
    assert_eq!(
        author_events.len(),
        1,
        "shared author should have one create event"
    );
    assert_eq!(
        author_events[0]["name"].as_str(),
        Some(shared_author.as_str())
    );
    assert_eq!(author_events[0]["operation"].as_str(), Some("create"));

    for book_id in &imported_book_ids {
        delete_test_book(book_id, &token).await?;
    }
    delete_test_author(shared_author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_import_books_empty_returns_error() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let import_query = r#"mutation { importBooks(books: []) { id } }"#;
    let (_, response) = graphql_request(import_query, Some(&token)).await?;
    assert!(
        response.get("errors").is_some(),
        "importBooks with empty list should return errors: {:?}",
        response.get("errors")
    );

    Ok(())
}
