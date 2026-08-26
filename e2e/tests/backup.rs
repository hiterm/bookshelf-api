use anyhow::{Context, Result};
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
async fn state_and_full_backup_round_trip_and_isolation() -> Result<()> {
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

    let state = get_backup(&client, &token_a, "state").await?;
    let full = get_backup(&client, &token_a, "full").await?;
    let user_b_before = get_backup(&client, &token_b, "full").await?;

    let extra_author = create_test_author("Temporary Author", &token_a).await?;
    create_test_book("Temporary Book", &extra_author, &token_a).await?;
    let before_state_restore_full = get_backup(&client, &token_a, "full").await?;
    assert_eq!(
        restore(&client, &token_a, "state", &state).await?,
        StatusCode::NO_CONTENT
    );
    let restored_state = get_backup(&client, &token_a, "state").await?;
    assert_eq!(restored_state["data"], state["data"]);
    let after_current_full = get_backup(&client, &token_a, "full").await?;
    let original_event_set_count = before_state_restore_full["history"]["eventSets"]
        .as_array()
        .unwrap()
        .len();
    let current_event_sets = after_current_full["history"]["eventSets"]
        .as_array()
        .unwrap();
    assert_eq!(current_event_sets.len(), original_event_set_count + 2);
    assert!(
        current_event_sets
            .iter()
            .rev()
            .take(2)
            .all(|event| event["operation"] == "snapshot_all")
    );
    let snapshot_extras = after_current_full["history"]["bookEvents"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|event| {
            (event["extra"]["reason"] == "state_backup_restore")
                .then(|| event["extra"]["phase"].as_str().unwrap())
        })
        .collect::<Vec<_>>();
    assert!(snapshot_extras.contains(&"before"));
    assert!(snapshot_extras.contains(&"after"));

    assert_eq!(
        restore(&client, &token_a, "full", &full).await?,
        StatusCode::NO_CONTENT
    );
    let restored_full = get_backup(&client, &token_a, "full").await?;
    assert_eq!(restored_full["data"], full["data"]);
    assert_eq!(history_shape(&restored_full), history_shape(&full));
    let restored_author_events = restored_full["history"]["authorEvents"].as_array().unwrap();
    let restored_ids = restored_author_events
        .iter()
        .map(|event| event["eventId"].as_i64().unwrap())
        .collect::<Vec<_>>();
    let remapped_source = restored_author_events
        .iter()
        .find(|event| event["operation"] == "restore")
        .context("restored Author history contains restore event")?["extra"]["source_event_id"]
        .as_i64()
        .unwrap();
    assert!(restored_ids.contains(&remapped_source));
    assert_ne!(remapped_source.to_string(), original_author_event_id);
    let user_b_after = get_backup(&client, &token_b, "full").await?;
    assert_eq!(user_b_after["data"], user_b_before["data"]);
    assert_eq!(history_shape(&user_b_after), history_shape(&user_b_before));

    assert_eq!(
        restore(&client, &token_b, "full", &full).await?,
        StatusCode::NO_CONTENT
    );
    let portable_restore = get_backup(&client, &token_b, "full").await?;
    assert_eq!(portable_restore["data"], full["data"]);
    assert_eq!(history_shape(&portable_restore), history_shape(&full));
    Ok(())
}

#[tokio::test]
#[serial]
async fn invalid_restore_is_atomic() -> Result<()> {
    let client = Client::new();
    let (_, token) = create_test_user().await?;
    let author = create_test_author("Atomic Author", &token).await?;
    create_test_book("Atomic Book", &author, &token).await?;
    let before = get_backup(&client, &token, "full").await?;
    let mut invalid = before.clone();
    invalid["version"] = json!(999);

    let response = client
        .post(format!("{}/backup/full/restore", get_server_url()?))
        .bearer_auth(&token)
        .json(&invalid)
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body: Value = response.json().await.context("validation response JSON")?;
    assert_eq!(body["error"], "invalid_backup");

    let after = get_backup(&client, &token, "full").await?;
    assert_eq!(after["data"], before["data"]);
    assert_eq!(history_shape(&after), history_shape(&before));
    Ok(())
}

#[tokio::test]
#[serial]
async fn state_restore_rejects_oversized_body() -> Result<()> {
    let client = Client::new();
    let (_, token) = create_test_user().await?;
    let response = client
        .post(format!("{}/backup/state/restore", get_server_url()?))
        .bearer_auth(token)
        .header("content-type", "application/json")
        .body(vec![b' '; 10 * 1024 * 1024 + 1])
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    Ok(())
}
