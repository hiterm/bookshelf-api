// E2E tests that run against a real Postgres instance.

#![cfg(test)]

use anyhow::Result;
use bookshelf_e2e::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn e2e_graphql_without_auth_returns_error() -> Result<()> {
    let query = r#"{ books { id title } }"#;
    let (status, _response) = graphql_request(query, None).await?;

    assert_eq!(
        status, 401,
        "Expected 401 Unauthorized when accessing GraphQL without authentication"
    );
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_register_user_and_logged_in_user() -> Result<()> {
    let user_id = uuid::Uuid::new_v4().to_string();
    let token = generate_test_token(&user_id)?;

    let (_, response) = graphql_request(r#"{ loggedInUser { id } }"#, Some(&token)).await?;
    assert_no_graphql_errors(&response, "loggedInUser before registration");
    assert!(
        response["data"]["loggedInUser"].is_null(),
        "unregistered user should not be returned by loggedInUser"
    );

    let (_, response) =
        graphql_request(r#"mutation { registerUser { id } }"#, Some(&token)).await?;
    assert_no_graphql_errors(&response, "registerUser");
    assert_eq!(
        response["data"]["registerUser"]["id"].as_str(),
        Some(user_id.as_str()),
        "registerUser should return the JWT subject"
    );

    let (_, response) = graphql_request(r#"{ loggedInUser { id } }"#, Some(&token)).await?;
    assert_no_graphql_errors(&response, "loggedInUser after registration");
    assert_eq!(
        response["data"]["loggedInUser"]["id"].as_str(),
        Some(user_id.as_str()),
        "loggedInUser should return the registered user"
    );

    Ok(())
}
