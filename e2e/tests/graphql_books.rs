// E2E tests that run against a real Postgres instance.

#![cfg(test)]

use anyhow::{Context, Result};
use bookshelf_e2e::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn e2e_graphql_books_empty() -> Result<()> {
    // Use a fresh user ID so the books list is always empty for this user
    let (_user_id, token) = create_test_user().await?;

    let query = r#"{ books { id title } }"#;
    let (_, response) = graphql_request(query, Some(&token)).await?;

    let data = response.get("data").context("data field must exist")?;
    let books = data.get("books").context("books field must exist")?;
    let books_array = books.as_array().context("books should be an array")?;
    assert!(
        books_array.is_empty(),
        "books should be empty for a new user"
    );
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_crud_book() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    // Create author first
    let author_name = format!("Test Author for CRUD {}", uuid::Uuid::new_v4());
    let create_author_query = format!(
        r#"mutation {{ createAuthor(authorData: {{ name: "{}" }}) {{ id }} }}"#,
        author_name
    );
    let (_, response) = graphql_request(&create_author_query, Some(&token)).await?;
    let data = response.get("data").context("data field must exist")?;
    let author_result = data
        .get("createAuthor")
        .context("createAuthor field must exist")?;
    let author_id = author_result
        .get("id")
        .context("id field must exist")?
        .as_str()
        .context("id must be string")?;

    // Verify author was created by fetching it
    let author_query = format!(r#"{{ author(id: "{}") {{ id name }} }}"#, author_id);
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
        author_name_from_query, author_name,
        "author name should match"
    );

    // Create book with author
    let create_query = format!(
        r#"
        mutation {{
            createBook(bookData: {{
                title: "Test Book"
                authorIds: ["{}"]
                isbn: "9783161484100"
                read: false
                owned: true
                priority: 1
                format: E_BOOK
                store: KINDLE
            }}) {{
                id
                title
                read
                owned
                priority
                createdAt
                updatedAt
            }}
        }}
        "#,
        author_id
    );
    let (_, response) = graphql_request(&create_query, Some(&token)).await?;
    let data = response.get("data").context("data field must exist")?;
    let create_result = data
        .get("createBook")
        .context("createBook field must exist")?;
    let book_id = create_result
        .get("id")
        .context("id field must exist")?
        .as_str()
        .context("id must be string")?;

    // Update book
    let update_result = update_test_book(
        book_id,
        "Updated Test Book",
        author_id,
        "9783161484100",
        true,
        true,
        2,
        "PRINTED",
        "KINDLE",
        &token,
    )
    .await?;
    assert_eq!(
        update_result
            .get("title")
            .context("title field must exist")?
            .as_str(),
        Some("Updated Test Book")
    );
    assert_eq!(
        update_result
            .get("read")
            .context("read field must exist")?
            .as_bool(),
        Some(true)
    );

    // Verify update by reading the book
    let query = format!(
        r#"{{ book(id: "{}") {{ id title read priority createdAt updatedAt authors {{ id name }} }} }}"#,
        book_id
    );
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    let data = response.get("data").context("data field must exist")?;
    let book = data.get("book").context("book field must exist")?;
    let created_at = book
        .get("createdAt")
        .context("createdAt field must exist")?
        .as_i64()
        .context("createdAt should be i64")?;
    let updated_at = book
        .get("updatedAt")
        .context("updatedAt field must exist")?
        .as_i64()
        .context("updatedAt should be i64")?;
    assert!(created_at > 0, "created_at should be positive");
    assert!(updated_at > 0, "updated_at should be positive");
    // Verify updated_at >= created_at (update happened after create)
    assert!(
        updated_at >= created_at,
        "updated_at should be >= created_at"
    );
    assert_eq!(
        book.get("title")
            .context("title field must exist")?
            .as_str(),
        Some("Updated Test Book")
    );
    assert_eq!(
        book.get("read").context("read field must exist")?.as_bool(),
        Some(true)
    );
    assert_eq!(
        book.get("priority")
            .context("priority field must exist")?
            .as_i64(),
        Some(2)
    );
    let authors = book
        .get("authors")
        .context("authors field must exist")?
        .as_array()
        .context("authors should be an array")?;
    assert_eq!(authors.len(), 1, "should have 1 author");
    let author_from_book = authors[0]
        .get("id")
        .context("author id field must exist")?
        .as_str()
        .context("author id should be string")?;
    assert_eq!(
        author_from_book, author_id,
        "author id should match the created author"
    );

    // Delete book
    delete_test_book(book_id, &token).await?;

    // Verify deletion
    let query = format!(r#"{{ book(id: "{}") {{ id title }} }}"#, book_id);
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    let data = response.get("data").context("data field must exist")?;
    let book = data.get("book").context("book field must exist")?;
    assert!(book.is_null(), "book should be null after deletion");

    // Clean up the author (book already deleted above)
    delete_test_author(author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_book_by_id() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    // Create author first
    let author_name = format!("Test Author for BookByID {}", uuid::Uuid::new_v4());
    let create_author_query = format!(
        r#"mutation {{ createAuthor(authorData: {{ name: "{}" }}) {{ id }} }}"#,
        author_name
    );
    let (_, response) = graphql_request(&create_author_query, Some(&token)).await?;
    let data = response.get("data").context("data field must exist")?;
    let author_result = data
        .get("createAuthor")
        .context("createAuthor field must exist")?;
    let author_id = author_result
        .get("id")
        .context("id field must exist")?
        .as_str()
        .context("id must be string")?;

    // Create book with author
    let create_query = format!(
        r#"
        mutation {{
            createBook(bookData: {{
                title: "Book By ID Test"
                authorIds: ["{}"]
                isbn: "9780123456789"
                read: false
                owned: true
                priority: 1
                format: E_BOOK
                store: KINDLE
            }}) {{
                id
                title
            }}
        }}
        "#,
        author_id
    );
    let (_, response) = graphql_request(&create_query, Some(&token)).await?;
    let data = response.get("data").context("data field must exist")?;
    let create_result = data
        .get("createBook")
        .context("createBook field must exist")?;
    let book_id = create_result
        .get("id")
        .context("id field must exist")?
        .as_str()
        .context("id must be string")?;

    // Get book by ID
    let query = format!(
        r#"{{ book(id: "{}") {{ id title isbn read owned priority format store }} }}"#,
        book_id
    );
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    let data = response.get("data").context("data field must exist")?;
    let book = data.get("book").context("book field must exist")?;
    assert_eq!(
        book.get("title")
            .context("title field must exist")?
            .as_str(),
        Some("Book By ID Test")
    );
    assert_eq!(
        book.get("isbn").context("isbn field must exist")?.as_str(),
        Some("9780123456789")
    );

    // Clean up: delete book first, then author
    delete_test_book(book_id, &token).await?;
    delete_test_author(author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_book_with_invalid_id() -> Result<()> {
    let user_id = uuid::Uuid::new_v4().to_string();
    let token = generate_test_token(&user_id)?;

    let query = r#"{ book(id: "00000000-0000-0000-0000-000000000000") { id title } }"#;
    let (_, response) = graphql_request(query, Some(&token)).await?;

    let data = response.get("data").context("data field must exist")?;
    let book = data.get("book").context("book field must exist")?;
    assert!(book.is_null(), "book with invalid ID should be null");
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_create_book_without_auth() -> Result<()> {
    let query = r#"
        mutation {
            createBook(bookData: {
                title: "Test Book"
                authorIds: []
                isbn: "9781234567890"
                read: false
                owned: true
                priority: 1
                format: E_BOOK
                store: KINDLE
            }) {
                id
            }
        }
    "#;
    let (status, _response) = graphql_request(query, None).await?;
    assert_eq!(status, 401, "createBook without auth should return 401");
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_update_nonexistent_book_returns_error() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;
    let nonexistent_id = uuid::Uuid::new_v4().to_string();
    let query = format!(
        r#"
        mutation {{
            updateBook(bookData: {{
                id: "{}"
                title: "Ghost Book"
                authorIds: []
                isbn: ""
                read: false
                owned: false
                priority: 50
                format: E_BOOK
                store: KINDLE
            }}) {{ id title }}
        }}
        "#,
        nonexistent_id
    );
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    assert_graphql_errors(&response, "updateBook for a non-existent book");
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_delete_nonexistent_book_returns_error() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;
    let nonexistent_id = uuid::Uuid::new_v4().to_string();
    let query = format!(r#"mutation {{ deleteBook(bookId: "{}") }}"#, nonexistent_id);
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    assert_graphql_errors(&response, "deleteBook for a non-existent book");
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_books_authors_are_user_isolated() -> Result<()> {
    let (_owner_user_id, owner_token) = create_test_user().await?;
    let (_other_user_id, other_token) = create_test_user().await?;

    let author_name = format!("Isolation Author {}", uuid::Uuid::new_v4());
    let author_id = create_test_author(&author_name, &owner_token).await?;
    let book_id = create_test_book("Isolation Book", &author_id, &owner_token).await?;

    let (_, response) =
        graphql_request(r#"{ books { id } authors { id } }"#, Some(&other_token)).await?;
    let other_books = response["data"]["books"]
        .as_array()
        .context("books should be an array")?;
    let other_authors = response["data"]["authors"]
        .as_array()
        .context("authors should be an array")?;
    assert!(
        other_books
            .iter()
            .all(|book| book["id"].as_str() != Some(book_id.as_str())),
        "other user should not list owner user's book"
    );
    assert!(
        other_authors
            .iter()
            .all(|author| author["id"].as_str() != Some(author_id.as_str())),
        "other user should not list owner user's author"
    );

    let book_query = format!(r#"{{ book(id: "{}") {{ id }} }}"#, book_id);
    let (_, response) = graphql_request(&book_query, Some(&other_token)).await?;
    assert!(
        response["data"]["book"].is_null(),
        "other user should not fetch owner user's book by id"
    );

    let author_query = format!(r#"{{ author(id: "{}") {{ id }} }}"#, author_id);
    let (_, response) = graphql_request(&author_query, Some(&other_token)).await?;
    assert!(
        response["data"]["author"].is_null(),
        "other user should not fetch owner user's author by id"
    );

    let update_book_query = format!(
        r#"
        mutation {{
            updateBook(bookData: {{
                id: "{}"
                title: "Hijacked Book"
                authorIds: []
                isbn: ""
                read: true
                owned: true
                priority: 1
                format: PRINTED
                store: KINDLE
            }}) {{ id }}
        }}
        "#,
        book_id
    );
    let (_, response) = graphql_request(&update_book_query, Some(&other_token)).await?;
    assert_graphql_errors(&response, "other user's updateBook");

    let delete_book_query = format!(r#"mutation {{ deleteBook(bookId: "{}") }}"#, book_id);
    let (_, response) = graphql_request(&delete_book_query, Some(&other_token)).await?;
    assert_graphql_errors(&response, "other user's deleteBook");

    let update_author_query = format!(
        r#"mutation {{ updateAuthor(authorData: {{ id: "{}", name: "Hijacked Author" }}) {{ id }} }}"#,
        author_id
    );
    let (_, response) = graphql_request(&update_author_query, Some(&other_token)).await?;
    assert_graphql_errors(&response, "other user's updateAuthor");

    let delete_author_query = format!(r#"mutation {{ deleteAuthor(authorId: "{}") }}"#, author_id);
    let (_, response) = graphql_request(&delete_author_query, Some(&other_token)).await?;
    assert_graphql_errors(&response, "other user's deleteAuthor");

    let owner_book_query = format!(r#"{{ book(id: "{}") {{ title }} }}"#, book_id);
    let (_, response) = graphql_request(&owner_book_query, Some(&owner_token)).await?;
    assert_eq!(
        response["data"]["book"]["title"].as_str(),
        Some("Isolation Book"),
        "owner user's book should remain unchanged"
    );
    let owner_author_query = format!(r#"{{ author(id: "{}") {{ name }} }}"#, author_id);
    let (_, response) = graphql_request(&owner_author_query, Some(&owner_token)).await?;
    assert_eq!(
        response["data"]["author"]["name"].as_str(),
        Some(author_name.as_str()),
        "owner user's author should remain unchanged"
    );

    delete_test_book(&book_id, &owner_token).await?;
    delete_test_author(&author_id, &owner_token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_create_mutations_validate_input() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let (_, response) = graphql_request(
        r#"mutation { createAuthor(authorData: { name: "" }) { id } }"#,
        Some(&token),
    )
    .await?;
    assert_graphql_errors(&response, "createAuthor with an empty name");

    let invalid_book_queries = [
        r#"
        mutation {
            createBook(bookData: {
                title: ""
                authorIds: []
                isbn: ""
                read: false
                owned: false
                priority: 50
                format: E_BOOK
                store: KINDLE
            }) { id }
        }
        "#,
        r#"
        mutation {
            createBook(bookData: {
                title: "Invalid ISBN Book"
                authorIds: []
                isbn: "bad-isbn"
                read: false
                owned: false
                priority: 50
                format: E_BOOK
                store: KINDLE
            }) { id }
        }
        "#,
        r#"
        mutation {
            createBook(bookData: {
                title: "Invalid Priority Book"
                authorIds: []
                isbn: ""
                read: false
                owned: false
                priority: 101
                format: E_BOOK
                store: KINDLE
            }) { id }
        }
        "#,
    ];
    for query in invalid_book_queries {
        let (_, response) = graphql_request(query, Some(&token)).await?;
        assert_graphql_errors(&response, "createBook with invalid input");
    }

    let (_, response) = graphql_request(
        r#"{ books { id } authors { id } eventSets { id } }"#,
        Some(&token),
    )
    .await?;
    assert!(
        response["data"]["books"]
            .as_array()
            .context("books should be an array")?
            .is_empty(),
        "invalid createBook requests should not create books"
    );
    assert!(
        response["data"]["authors"]
            .as_array()
            .context("authors should be an array")?
            .is_empty(),
        "invalid createAuthor request should not create authors"
    );
    assert!(
        response["data"]["eventSets"]
            .as_array()
            .context("eventSets should be an array")?
            .is_empty(),
        "invalid create requests should not create event sets"
    );

    Ok(())
}
