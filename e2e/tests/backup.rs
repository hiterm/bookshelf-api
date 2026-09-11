#![cfg(test)]

use anyhow::{Context, Result};
use bookshelf_e2e::{
    assert_no_graphql_errors, create_test_author, create_test_book, create_test_book_with_event,
    create_test_user, delete_test_author, delete_test_book, get_server_url, graphql_request,
};
use reqwest::{Client, StatusCode, header};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn snapshot_and_full_are_authenticated_json_attachments() -> Result<()> {
    let (_, token) = create_test_user().await?;
    let author_id = create_test_author("backup author", &token).await?;
    let book_id = create_test_book("backup book", &author_id, &token).await?;
    let client = Client::new();
    let base_url = get_server_url()?;

    for scope in ["snapshot", "full"] {
        let response = client
            .get(format!("{base_url}/backup/{scope}"))
            .bearer_auth(&token)
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
            .to_str()?;
        assert!(
            disposition.starts_with(&format!("attachment; filename=\"bookshelf-backup-{scope}-"))
        );
        assert!(disposition.ends_with("Z.json\""));
        assert!(!disposition.contains(':'));

        let body: serde_json::Value = response.json().await?;
        assert_eq!(body["format"], "bookshelf-backup");
        assert_eq!(body["version"], 1);
        assert_eq!(body["scope"], scope);
        assert!(
            body["exportedAt"]
                .as_str()
                .is_some_and(|value| value.ends_with('Z'))
        );
        assert!(
            body["data"]["authors"]
                .as_array()
                .is_some_and(|values| { values.iter().any(|author| author["id"] == author_id) })
        );
        assert!(
            body["data"]["books"]
                .as_array()
                .is_some_and(|values| { values.iter().any(|book| book["id"] == book_id) })
        );
        assert!(!body.to_string().contains("user_id"));
        if scope == "snapshot" {
            assert!(body["data"].get("history").is_none());
        } else {
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
    Ok(())
}

#[tokio::test]
async fn backup_without_authentication_is_rejected() -> Result<()> {
    let response = Client::new()
        .get(format!("{}/backup/snapshot", get_server_url()?))
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    Ok(())
}

#[tokio::test]
#[serial]
async fn full_backup_is_tenant_isolated_at_the_http_boundary() -> Result<()> {
    let (_, token_a) = create_test_user().await?;
    let author_a = create_test_author("tenant A backup author", &token_a).await?;
    let book_a = create_test_book("tenant A backup book", &author_a, &token_a).await?;
    let (_, token_b) = create_test_user().await?;
    let author_b = create_test_author("tenant B backup author", &token_b).await?;
    let book_b = create_test_book("tenant B backup book", &author_b, &token_b).await?;

    let response = Client::new()
        .get(format!("{}/backup/full", get_server_url()?))
        .bearer_auth(&token_a)
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = response.json().await?;
    let serialized = body.to_string();

    assert!(serialized.contains(&author_a));
    assert!(serialized.contains(&book_a));
    assert!(!serialized.contains(&author_b));
    assert!(!serialized.contains(&book_b));
    assert!(!serialized.contains("tenant B backup author"));
    assert!(!serialized.contains("tenant B backup book"));

    delete_test_book(&book_a, &token_a).await?;
    delete_test_author(&author_a, &token_a).await?;
    delete_test_book(&book_b, &token_b).await?;
    delete_test_author(&author_b, &token_b).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn full_backup_contains_history_generated_by_normal_writes() -> Result<()> {
    let (_, token) = create_test_user().await?;
    let author_id = create_test_author("vertical backup author", &token).await?;
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

    let response = Client::new()
        .get(format!("{}/backup/full", get_server_url()?))
        .bearer_auth(&token)
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = response.json().await?;
    let data = &body["data"];

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
    let book_change = update_operation["changes"]["books"]
        .as_array()
        .context("update Book changes")?
        .iter()
        .find(|change| change["bookId"] == book_id)
        .context("update Book change")?;
    assert_eq!(book_change["beforeRevisionNumber"], 1);
    assert_eq!(book_change["afterRevisionNumber"], 2);

    delete_test_book(&book_id, &token).await?;
    delete_test_author(&author_id, &token).await?;
    Ok(())
}
