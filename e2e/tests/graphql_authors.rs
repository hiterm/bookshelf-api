// E2E tests that run against a real Postgres instance.

#![cfg(test)]

use anyhow::{Context, Result};
use bookshelf_e2e::*;
use serial_test::serial;

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
async fn e2e_graphql_create_author() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    // Use a random name to keep the author unique across runs.
    let random_name = format!("Test Author {}", uuid::Uuid::new_v4());

    let query = format!(
        r#"mutation {{ createAuthor(authorData: {{ name: "{}" }}) {{ author {{ id name yomi }} eventSetId }} }}"#,
        random_name
    );
    let (_, response) = graphql_request(&query, Some(&token)).await?;

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
    assert_eq!(create_result["yomi"].as_str(), Some(""));

    // Verify author was created by fetching it
    let author_query = format!(r#"{{ author(id: "{}") {{ id name yomi }} }}"#, author_id);
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
    assert_eq!(author["yomi"].as_str(), Some(""));

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

    // Create book associated with the author
    let book_id = create_test_book("Book For Author Update Test", &author_id, &token).await?;

    // Update author name while the author has an associated book
    let updated_name = format!("Author After Update {}", uuid::Uuid::new_v4());
    let update_query = format!(
        r#"mutation {{ updateAuthor(authorData: {{ id: "{}", name: "{}" }}) {{ author {{ id name }} eventSetId }} }}"#,
        author_id, updated_name
    );
    let (_, response) = graphql_request(&update_query, Some(&token)).await?;
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

    // Verify update by fetching the author
    let query = format!(r#"{{ author(id: "{}") {{ id name }} }}"#, author_id);
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    assert_eq!(
        response["data"]["author"]["name"].as_str(),
        Some(updated_name.as_str()),
        "author name should reflect the update"
    );

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
        r#"mutation {{ updateAuthor(authorData: {{ id: "{}", name: "Ghost" }}) {{ author {{ id name }} eventSetId }} }}"#,
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
