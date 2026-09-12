#![cfg(test)]

use anyhow::{Context, Result};
use bookshelf_e2e::{
    assert_no_graphql_errors, create_test_author, create_test_author_with_event, create_test_book,
    create_test_book_with_event, create_test_user, delete_test_author, delete_test_book,
    get_server_url, graphql_request,
};
use reqwest::{Client, StatusCode, header};
use serial_test::serial;
use std::collections::BTreeSet;

fn assert_exact_keys(value: &serde_json::Value, expected: &[&str], context: &str) -> Result<()> {
    let actual = value
        .as_object()
        .with_context(|| format!("{context} should be an object"))?
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(actual, expected, "unexpected keys in {context}");
    Ok(())
}

fn assert_common_backup_schema(body: &serde_json::Value, scope: &str) -> Result<()> {
    assert_exact_keys(
        body,
        &["format", "version", "scope", "exportedAt", "data"],
        "backup envelope",
    )?;
    assert_eq!(body["format"], "bookshelf-backup");
    assert_eq!(body["version"], 1);
    assert_eq!(body["scope"], scope);
    assert!(!body.to_string().contains("user_id"));
    Ok(())
}

fn assert_author_schema(author: &serde_json::Value) -> Result<()> {
    assert_exact_keys(
        author,
        &["id", "name", "yomi", "createdAt", "updatedAt"],
        "backup author",
    )
}

fn assert_book_schema(book: &serde_json::Value) -> Result<()> {
    assert_exact_keys(
        book,
        &[
            "id",
            "title",
            "authorIds",
            "isbn",
            "read",
            "owned",
            "priority",
            "format",
            "store",
            "purchaseDate",
            "createdAt",
            "updatedAt",
        ],
        "backup book",
    )
}

fn assert_full_history_schema(history: &serde_json::Value) -> Result<()> {
    assert_exact_keys(
        history,
        &["operations", "bookRevisions", "authorRevisions"],
        "backup history",
    )?;
    for operation in history["operations"]
        .as_array()
        .context("history.operations")?
    {
        assert_exact_keys(
            operation,
            &[
                "id",
                "type",
                "detail",
                "undoOfOperationId",
                "createdAt",
                "changes",
            ],
            "backup operation",
        )?;
        assert_exact_keys(
            &operation["changes"],
            &["books", "authors"],
            "operation changes",
        )?;
        for change in operation["changes"]["books"]
            .as_array()
            .context("operation book changes")?
        {
            assert_exact_keys(
                change,
                &["bookId", "beforeRevisionNumber", "afterRevisionNumber"],
                "backup book change",
            )?;
        }
        for change in operation["changes"]["authors"]
            .as_array()
            .context("operation author changes")?
        {
            assert_exact_keys(
                change,
                &["authorId", "beforeRevisionNumber", "afterRevisionNumber"],
                "backup author change",
            )?;
        }
    }
    for revision in history["bookRevisions"]
        .as_array()
        .context("history.bookRevisions")?
    {
        assert_exact_keys(
            revision,
            &["bookId", "revisionNumber", "snapshot", "recordedAt"],
            "backup book revision",
        )?;
        assert_exact_keys(
            &revision["snapshot"],
            &[
                "title",
                "authorIds",
                "isbn",
                "read",
                "owned",
                "priority",
                "format",
                "store",
                "purchaseDate",
                "createdAt",
                "updatedAt",
            ],
            "book revision snapshot",
        )?;
    }
    for revision in history["authorRevisions"]
        .as_array()
        .context("history.authorRevisions")?
    {
        assert_exact_keys(
            revision,
            &["authorId", "revisionNumber", "snapshot", "recordedAt"],
            "backup author revision",
        )?;
        assert_exact_keys(
            &revision["snapshot"],
            &["name", "yomi", "createdAt", "updatedAt"],
            "author revision snapshot",
        )?;
    }
    Ok(())
}

async fn backup_response(token: &str, scope: &str) -> Result<serde_json::Value> {
    let response = Client::new()
        .get(format!("{}/backup/{scope}", get_server_url()?))
        .bearer_auth(token)
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("application/json")
    );
    assert_eq!(
        response
            .headers()
            .get(header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    let disposition = response
        .headers()
        .get(header::CONTENT_DISPOSITION)
        .context("content-disposition")?
        .to_str()?
        .to_owned();
    let body: serde_json::Value = response.json().await?;
    let exported_at = body["exportedAt"]
        .as_str()
        .context("exportedAt should be a string")?;
    let exported_second = exported_at
        .trim_end_matches('Z')
        .split('.')
        .next()
        .context("exportedAt should contain a timestamp")?
        .replace(':', "");
    assert_eq!(
        disposition,
        format!("attachment; filename=\"bookshelf-backup-{scope}-{exported_second}Z.json\"")
    );
    Ok(body)
}

#[tokio::test]
#[serial]
async fn snapshot_and_full_are_authenticated_json_attachments() -> Result<()> {
    let (_, token) = create_test_user().await?;
    let author_id = create_test_author("backup author", &token).await?;
    let create_book = format!(
        r#"mutation {{ createBook(bookData: {{
          title: "backup book", authorIds: ["{author_id}"], isbn: "9783161484100",
          read: true, owned: true, priority: 7, format: PRINTED, store: KINDLE,
          purchaseDate: "2020-05-01"
        }}) {{ book {{ id }} }} }}"#
    );
    let (_, create_response) = graphql_request(&create_book, Some(&token)).await?;
    assert_no_graphql_errors(&create_response, "create representative backup Book");
    let book_id = create_response["data"]["createBook"]["book"]["id"]
        .as_str()
        .context("created Book id")?
        .to_owned();

    for scope in ["snapshot", "full"] {
        let body = backup_response(&token, scope).await?;
        assert_common_backup_schema(&body, scope)?;
        let author = body["data"]["authors"]
            .as_array()
            .context("data.authors")?
            .iter()
            .find(|author| author["id"] == author_id)
            .context("created Author in backup")?;
        assert_author_schema(author)?;
        assert_eq!(author["name"], "backup author");
        assert_eq!(author["yomi"], "");
        let book = body["data"]["books"]
            .as_array()
            .context("data.books")?
            .iter()
            .find(|book| book["id"] == book_id)
            .context("created Book in backup")?;
        assert_book_schema(book)?;
        assert_eq!(book["title"], "backup book");
        assert_eq!(book["authorIds"], serde_json::json!([author_id]));
        assert_eq!(book["isbn"], "9783161484100");
        assert_eq!(book["read"], true);
        assert_eq!(book["owned"], true);
        assert_eq!(book["priority"], 7);
        assert_eq!(book["format"], "PRINTED");
        assert_eq!(book["store"], "KINDLE");
        assert_eq!(book["purchaseDate"], "2020-05-01");
        if scope == "snapshot" {
            assert_exact_keys(&body["data"], &["authors", "books"], "snapshot data")?;
        } else {
            assert_exact_keys(&body["data"], &["authors", "books", "history"], "full data")?;
            assert!(
                body["data"]["history"]["operations"]
                    .as_array()
                    .is_some_and(|values| !values.is_empty())
            );
            assert!(
                body["data"]["history"]["bookRevisions"]
                    .as_array()
                    .is_some_and(|values| !values.is_empty())
            );
            assert!(
                body["data"]["history"]["authorRevisions"]
                    .as_array()
                    .is_some_and(|values| !values.is_empty())
            );
        }
    }
    delete_test_book(&book_id, &token).await?;
    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
async fn backup_without_authentication_is_rejected() -> Result<()> {
    for scope in ["snapshot", "full"] {
        let response = Client::new()
            .get(format!("{}/backup/{scope}", get_server_url()?))
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{scope}");
    }
    Ok(())
}

struct TenantIsolationFixture {
    author_b: String,
    author_b_revision: i32,
    author_b_operation: String,
    book_b: String,
    book_b_revision: i32,
    book_b_operation: String,
}

async fn assert_backup_is_tenant_isolated(
    endpoint: &str,
) -> Result<(serde_json::Value, TenantIsolationFixture)> {
    let (_, token_a) = create_test_user().await?;
    let author_a = create_test_author("tenant A backup author", &token_a).await?;
    let book_a = create_test_book("tenant A backup book", &author_a, &token_a).await?;
    let (_, token_b) = create_test_user().await?;
    let (author_b, author_b_revision, author_b_operation) =
        create_test_author_with_event("tenant B backup author", &token_b).await?;
    let (book_b, book_b_revision, book_b_operation) =
        create_test_book_with_event("tenant B backup book", &author_b, &token_b).await?;

    let response = Client::new()
        .get(format!("{}{endpoint}", get_server_url()?))
        .bearer_auth(&token_a)
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = response.json().await?;
    let authors = body["data"]["authors"].as_array().context("data.authors")?;
    let books = body["data"]["books"].as_array().context("data.books")?;

    assert!(authors.iter().any(|author| author["id"] == author_a));
    assert!(books.iter().any(|book| book["id"] == book_a));
    assert!(!authors.iter().any(|author| author["id"] == author_b));
    assert!(!books.iter().any(|book| book["id"] == book_b));

    let serialized = body.to_string();
    assert!(!serialized.contains(&author_b));
    assert!(!serialized.contains(&book_b));
    assert!(!serialized.contains("tenant B backup author"));
    assert!(!serialized.contains("tenant B backup book"));

    delete_test_book(&book_a, &token_a).await?;
    delete_test_author(&author_a, &token_a).await?;
    delete_test_book(&book_b, &token_b).await?;
    delete_test_author(&author_b, &token_b).await?;
    Ok((
        body,
        TenantIsolationFixture {
            author_b,
            author_b_revision,
            author_b_operation,
            book_b,
            book_b_revision,
            book_b_operation,
        },
    ))
}

#[tokio::test]
#[serial]
async fn snapshot_backup_is_tenant_isolated_at_the_http_boundary() -> Result<()> {
    assert_backup_is_tenant_isolated("/backup/snapshot")
        .await
        .map(|_| ())
}

#[tokio::test]
#[serial]
async fn full_backup_is_tenant_isolated_at_the_http_boundary() -> Result<()> {
    let (body, fixture) = assert_backup_is_tenant_isolated("/backup/full").await?;
    let history = &body["data"]["history"];
    let operations = history["operations"]
        .as_array()
        .context("data.history.operations")?;
    let book_revisions = history["bookRevisions"]
        .as_array()
        .context("data.history.bookRevisions")?;
    let author_revisions = history["authorRevisions"]
        .as_array()
        .context("data.history.authorRevisions")?;

    assert!(operations.iter().all(|operation| {
        operation["id"] != fixture.author_b_operation && operation["id"] != fixture.book_b_operation
    }));
    assert!(book_revisions.iter().all(|revision| {
        revision["bookId"] != fixture.book_b
            || revision["revisionNumber"] != fixture.book_b_revision
    }));
    assert!(author_revisions.iter().all(|revision| {
        revision["authorId"] != fixture.author_b
            || revision["revisionNumber"] != fixture.author_b_revision
    }));
    assert!(operations.iter().all(|operation| {
        operation["changes"]["books"]
            .as_array()
            .is_some_and(|changes| {
                changes
                    .iter()
                    .all(|change| change["bookId"] != fixture.book_b)
            })
    }));
    assert!(operations.iter().all(|operation| {
        operation["changes"]["authors"]
            .as_array()
            .is_some_and(|changes| {
                changes
                    .iter()
                    .all(|change| change["authorId"] != fixture.author_b)
            })
    }));
    Ok(())
}

#[tokio::test]
#[serial]
async fn full_backup_contains_history_generated_by_normal_writes() -> Result<()> {
    let (_, token) = create_test_user().await?;
    let (author_id, author_revision, author_operation_id) =
        create_test_author_with_event("vertical backup author", &token).await?;
    let (book_id, create_revision, _) =
        create_test_book_with_event("Before backup update", &author_id, &token).await?;
    let update = format!(
        r#"mutation {{ updateBook(bookData: {{
          id: "{book_id}", title: "After backup update", authorIds: ["{author_id}"],
          isbn: "", read: true, owned: false, priority: 50,
          format: E_BOOK, store: KINDLE
        }}) {{ revisionNumber operationId }} }}"#
    );
    let (_, update_response) = graphql_request(&update, Some(&token)).await?;
    assert_no_graphql_errors(&update_response, "update Book before backup");
    let update_payload = &update_response["data"]["updateBook"];
    let update_revision = update_payload["revisionNumber"]
        .as_i64()
        .context("update revision number")?;
    let update_operation_id = update_payload["operationId"]
        .as_str()
        .context("update operation id")?;
    assert_eq!(create_revision, 1);
    assert_eq!(update_revision, 2);
    assert_eq!(author_revision, 1);

    let response = Client::new()
        .get(format!("{}/backup/full", get_server_url()?))
        .bearer_auth(&token)
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = response.json().await?;
    let data = &body["data"];
    assert_common_backup_schema(&body, "full")?;
    assert_exact_keys(data, &["authors", "books", "history"], "full data")?;
    for author in data["authors"].as_array().context("current Authors")? {
        assert_author_schema(author)?;
    }
    for book in data["books"].as_array().context("current Books")? {
        assert_book_schema(book)?;
    }
    assert_full_history_schema(&data["history"])?;

    let current_book = data["books"]
        .as_array()
        .context("current Books")?
        .iter()
        .find(|book| book["id"] == book_id)
        .context("updated current Book")?;
    assert_eq!(current_book["title"], "After backup update");
    assert_eq!(current_book["read"], true);

    let revisions = data["history"]["bookRevisions"]
        .as_array()
        .context("Book revisions")?
        .iter()
        .filter(|revision| revision["bookId"] == book_id)
        .collect::<Vec<_>>();
    assert_eq!(revisions.len(), 2);
    assert_eq!(revisions[0]["revisionNumber"], 1);
    assert_eq!(revisions[0]["snapshot"]["title"], "Before backup update");
    assert_eq!(
        revisions[0]["snapshot"]["authorIds"],
        serde_json::json!([author_id])
    );
    assert_eq!(revisions[1]["revisionNumber"], 2);
    assert_eq!(revisions[1]["snapshot"]["title"], "After backup update");
    assert_eq!(
        revisions[1]["snapshot"]["authorIds"],
        serde_json::json!([author_id])
    );

    let update_operation = data["history"]["operations"]
        .as_array()
        .context("Operations")?
        .iter()
        .find(|operation| operation["id"] == update_operation_id)
        .context("update Operation")?;
    assert_eq!(update_operation["type"], "update_book");
    let book_change = update_operation["changes"]["books"]
        .as_array()
        .context("update Book changes")?
        .iter()
        .find(|change| change["bookId"] == book_id)
        .context("update Book change")?;
    assert_eq!(book_change["beforeRevisionNumber"], 1);
    assert_eq!(book_change["afterRevisionNumber"], 2);

    let author_operation = data["history"]["operations"]
        .as_array()
        .context("Operations")?
        .iter()
        .find(|operation| operation["id"] == author_operation_id)
        .context("create Author Operation")?;
    assert_eq!(author_operation["type"], "create_author");
    let author_change = author_operation["changes"]["authors"]
        .as_array()
        .context("create Author changes")?
        .iter()
        .find(|change| change["authorId"] == author_id)
        .context("create Author change")?;
    assert!(author_change["beforeRevisionNumber"].is_null());
    assert_eq!(author_change["afterRevisionNumber"], 1);
    let author_revision = data["history"]["authorRevisions"]
        .as_array()
        .context("Author revisions")?
        .iter()
        .find(|revision| revision["authorId"] == author_id)
        .context("created Author revision")?;
    assert_eq!(author_revision["revisionNumber"], 1);
    assert_eq!(
        author_revision["snapshot"]["name"],
        "vertical backup author"
    );
    assert_eq!(author_revision["snapshot"]["yomi"], "");

    delete_test_book(&book_id, &token).await?;
    delete_test_author(&author_id, &token).await?;
    Ok(())
}
