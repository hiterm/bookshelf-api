use anyhow::Result;
use bookshelf_e2e::{
    create_test_author, create_test_author_with_event, create_test_book, create_test_user,
    get_server_url, graphql_request,
};
use reqwest::{Client, StatusCode};
use serde_json::{Value, json};
use serial_test::serial;

async fn get_backup(client: &Client, token: &str, kind: &str) -> Result<Value> {
    Ok(client
        .get(format!("{}/backup/{kind}", get_server_url()?))
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?)
}

async fn restore(client: &Client, token: &str, kind: &str, backup: &Value) -> Result<StatusCode> {
    Ok(client
        .post(format!("{}/backup/{kind}/restore", get_server_url()?))
        .bearer_auth(token)
        .json(backup)
        .send()
        .await?
        .status())
}

async fn validate_snapshot(client: &Client, token: &str, backup: &Value) -> Result<Value> {
    Ok(client
        .post(format!("{}/backup/snapshot/validate", get_server_url()?))
        .bearer_auth(token)
        .json(backup)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?)
}

fn history_shape(backup: &Value) -> Value {
    let history = &backup["history"];
    json!({
        "eventSets": history["eventSets"].as_array().unwrap().iter()
            .map(|event| event["operation"].clone()).collect::<Vec<_>>(),
        "bookEvents": history["bookEvents"].as_array().unwrap().iter()
            .map(|event| json!({"operation":event["operation"],"bookId":event["bookId"],"title":event["title"],"authorIds":event["authorIds"]})).collect::<Vec<_>>(),
        "authorEvents": history["authorEvents"].as_array().unwrap().iter()
            .map(|event| json!({"operation":event["operation"],"authorId":event["authorId"],"name":event["name"]})).collect::<Vec<_>>()
    })
}

#[tokio::test]
#[serial]
async fn snapshot_restore_and_full_export_are_isolated() -> Result<()> {
    let client = Client::new();
    let (_, token_a) = create_test_user().await?;
    let (_, token_b) = create_test_user().await?;
    let (author_a, original_author_event_id, _) =
        create_test_author_with_event("Backup Author A", &token_a).await?;
    create_test_book("Backup Book A", &author_a, &token_a).await?;
    let restore_query = format!(
        "mutation {{ restoreAuthor(eventId: \"{original_author_event_id}\") {{ author {{ id }} }} }}"
    );
    let (_, restore_response) = graphql_request(&restore_query, Some(&token_a)).await?;
    assert!(restore_response.get("errors").is_none());
    let author_b = create_test_author("Backup Author B", &token_b).await?;
    create_test_book("Backup Book B", &author_b, &token_b).await?;

    let snapshot = get_backup(&client, &token_a, "snapshot").await?;
    let full = get_backup(&client, &token_a, "full").await?;
    let user_b_before = get_backup(&client, &token_b, "full").await?;
    assert_eq!(full["data"], snapshot["data"]);
    assert!(!full["history"]["eventSets"].as_array().unwrap().is_empty());
    assert!(
        full["history"]["bookEvents"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| {
                event["authorIds"]
                    .as_array()
                    .is_some_and(|ids| ids.iter().any(|id| id == &author_a))
            })
    );
    assert!(!serde_json::to_string(&full)?.contains(&author_b));
    let validation = validate_snapshot(&client, &token_a, &snapshot).await?;
    assert_eq!(validation["valid"], true);
    assert_eq!(validation["summary"]["books"], 1);
    let after_validation = get_backup(&client, &token_a, "full").await?;
    assert_eq!(after_validation["data"], full["data"]);
    assert_eq!(history_shape(&after_validation), history_shape(&full));

    let extra_author = create_test_author("Temporary Author", &token_a).await?;
    create_test_book("Temporary Book", &extra_author, &token_a).await?;
    let before_snapshot_restore_full = get_backup(&client, &token_a, "full").await?;
    assert_eq!(
        restore(&client, &token_a, "snapshot", &snapshot).await?,
        StatusCode::NO_CONTENT
    );
    let restored_snapshot = get_backup(&client, &token_a, "snapshot").await?;
    assert_eq!(restored_snapshot["data"], snapshot["data"]);
    let after_snapshot_full = get_backup(&client, &token_a, "full").await?;
    let original_event_set_count = before_snapshot_restore_full["history"]["eventSets"]
        .as_array()
        .unwrap()
        .len();
    let snapshot_event_sets = after_snapshot_full["history"]["eventSets"]
        .as_array()
        .unwrap();
    assert_eq!(snapshot_event_sets.len(), original_event_set_count + 2);
    assert!(
        snapshot_event_sets
            .iter()
            .rev()
            .take(2)
            .all(|event| event["operation"] == "snapshot_all")
    );
    let snapshot_extras = after_snapshot_full["history"]["bookEvents"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|event| {
            (event["extra"]["reason"] == "snapshot_backup_restore")
                .then(|| event["extra"]["phase"].as_str().unwrap())
        })
        .collect::<Vec<_>>();
    assert!(snapshot_extras.contains(&"before"));
    assert!(snapshot_extras.contains(&"after"));
    let user_b_after = get_backup(&client, &token_b, "full").await?;
    assert_eq!(user_b_after["data"], user_b_before["data"]);
    assert_eq!(history_shape(&user_b_after), history_shape(&user_b_before));
    Ok(())
}

#[tokio::test]
#[serial]
async fn invalid_snapshot_restore_is_atomic() -> Result<()> {
    let client = Client::new();
    let (_, token) = create_test_user().await?;
    let author = create_test_author("Atomic Author", &token).await?;
    create_test_book("Atomic Book", &author, &token).await?;
    let before = get_backup(&client, &token, "full").await?;
    let mut invalid = get_backup(&client, &token, "snapshot").await?;
    invalid["version"] = json!(999);
    let validation = validate_snapshot(&client, &token, &invalid).await?;
    assert_eq!(validation["valid"], false);
    assert_eq!(validation["errors"][0]["code"], "unsupported_version");

    let mut duplicate = get_backup(&client, &token, "snapshot").await?;
    let author = duplicate["data"]["authors"][0].clone();
    duplicate["data"]["authors"]
        .as_array_mut()
        .unwrap()
        .push(author);
    assert_eq!(
        validate_snapshot(&client, &token, &duplicate).await?["errors"][0]["code"],
        "duplicate_author_id"
    );

    let mut missing_reference = get_backup(&client, &token, "snapshot").await?;
    missing_reference["data"]["books"][0]["authorIds"] =
        json!(["cccccccc-cccc-4ccc-8ccc-cccccccccccc"]);
    assert_eq!(
        validate_snapshot(&client, &token, &missing_reference).await?["errors"][0]["code"],
        "missing_author_reference"
    );

    let response = client
        .post(format!("{}/backup/snapshot/restore", get_server_url()?))
        .bearer_auth(&token)
        .json(&invalid)
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body: Value = response.json().await?;
    assert_eq!(body, validation);

    let after = get_backup(&client, &token, "full").await?;
    assert_eq!(after["data"], before["data"]);
    assert_eq!(history_shape(&after), history_shape(&before));
    Ok(())
}

#[tokio::test]
#[serial]
async fn snapshot_input_routes_enforce_authentication_and_size_limit() -> Result<()> {
    let client = Client::new();
    let (_, token) = create_test_user().await?;
    for action in ["validate", "restore"] {
        let response = client
            .post(format!("{}/backup/snapshot/{action}", get_server_url()?))
            .bearer_auth(&token)
            .header("content-type", "application/json")
            .body(vec![b' '; 10 * 1024 * 1024 + 1])
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }
    let unauthenticated = client
        .post(format!("{}/backup/snapshot/validate", get_server_url()?))
        .json(&json!({}))
        .send()
        .await?;
    assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);
    for action in ["validate", "restore"] {
        let absent = client
            .post(format!("{}/backup/full/{action}", get_server_url()?))
            .bearer_auth(&token)
            .json(&json!({}))
            .send()
            .await?;
        assert_eq!(absent.status(), StatusCode::NOT_FOUND);
    }
    Ok(())
}
