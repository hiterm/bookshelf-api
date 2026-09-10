#![cfg(test)]

use anyhow::{Context, Result};
use bookshelf_e2e::{create_test_author, create_test_book, create_test_user, get_server_url};
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
