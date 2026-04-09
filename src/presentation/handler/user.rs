//! GET /me endpoint - Returns authenticated user information
//!
//! This module provides the handler for retrieving the current authenticated
//! user's information from JWT claims.

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::presentation::{app_state::AppState, extractor::claims::Claims};

/// Response payload for GET /me endpoint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MeResponse {
    /// User ID from JWT subject claim
    pub id: String,
}

/// Handler for GET /me endpoint
///
/// Returns the authenticated user's information extracted from JWT claims.
/// The Claims extractor handles authentication; if no valid token is provided,
/// it returns 401 Unauthorized automatically.
pub async fn me_handler(claims: Claims, _state: State<Arc<AppState>>) -> Json<MeResponse> {
    Json(MeResponse { id: claims.sub })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::app_state::AppState;
    use crate::presentation::extractor::claims::JwtConfig;
    use moka::future::Cache;
    use std::collections::HashSet;
    use std::sync::Arc;
    use std::time::Duration;

    fn make_test_state() -> State<Arc<AppState>> {
        let jwks_cache = Cache::builder()
            .max_capacity(1)
            .time_to_live(Duration::from_hours(1))
            .build();
        State(Arc::new(AppState {
            jwt_config: JwtConfig {
                audience: "test".to_string(),
                domain: "test-issuer.local".to_string(),
            },
            jwks_cache,
        }))
    }

    // ============================================
    // Given-When-Then Structure
    // ============================================

    #[test]
    fn me_response_serialization_with_valid_user_id() {
        // Given: A MeResponse with a valid user ID
        let response = MeResponse {
            id: "auth0|123456789".to_string(),
        };

        // When: Serializing to JSON
        let json = serde_json::to_string(&response).unwrap();

        // Then: Should produce expected JSON structure
        assert_eq!(json, r#"{"id":"auth0|123456789"}"#);
    }

    #[test]
    fn me_response_deserialization_with_valid_user_id() {
        // Given: A valid JSON string
        let json = r#"{"id":"auth0|123456789"}"#;

        // When: Deserializing to MeResponse
        let response: MeResponse = serde_json::from_str(json).unwrap();

        // Then: Should have correct user ID
        assert_eq!(response.id, "auth0|123456789");
    }

    #[test]
    fn me_response_with_empty_user_id() {
        // Given: A MeResponse with empty user ID
        let response = MeResponse { id: "".to_string() };

        // When: Serializing to JSON
        let json = serde_json::to_string(&response).unwrap();

        // Then: Should serialize empty string correctly
        assert_eq!(json, r#"{"id":""}"#);
    }

    #[test]
    fn me_response_equality_same_id() {
        // Given: Two MeResponse instances with same ID
        let response1 = MeResponse {
            id: "auth0|abc123".to_string(),
        };
        let response2 = MeResponse {
            id: "auth0|abc123".to_string(),
        };

        // When: Comparing for equality
        // Then: Should be equal
        assert_eq!(response1, response2);
    }

    #[test]
    fn me_response_inequality_different_id() {
        // Given: Two MeResponse instances with different IDs
        let response1 = MeResponse {
            id: "auth0|abc123".to_string(),
        };
        let response2 = MeResponse {
            id: "auth0|def456".to_string(),
        };

        // When: Comparing for equality
        // Then: Should not be equal
        assert_ne!(response1, response2);
    }

    // ============================================
    // Claims to Response Mapping Tests
    // ============================================

    #[test]
    fn claims_with_sub_maps_to_response_id() {
        // Given: Claims with a subject
        let claims = Claims {
            sub: "auth0|user123".to_string(),
            _permissions: None,
        };

        // When: Creating response from claims
        let response = MeResponse { id: claims.sub };

        // Then: Response ID should match claims subject
        assert_eq!(response.id, "auth0|user123");
    }

    #[test]
    fn claims_with_permissions_preserves_sub() {
        // Given: Claims with permissions
        let mut permissions = HashSet::new();
        permissions.insert("read:books".to_string());
        permissions.insert("write:books".to_string());

        let claims = Claims {
            sub: "auth0|admin456".to_string(),
            _permissions: Some(permissions),
        };

        // When: Creating response from claims
        let response = MeResponse { id: claims.sub };

        // Then: Response ID should match claims subject (permissions not included)
        assert_eq!(response.id, "auth0|admin456");
    }

    #[test]
    fn claims_with_special_characters_in_sub() {
        // Given: Claims with special characters in subject
        let claims = Claims {
            sub: "auth0|user@example.com".to_string(),
            _permissions: None,
        };

        // When: Serializing response
        let response = MeResponse { id: claims.sub };
        let json = serde_json::to_string(&response).unwrap();

        // Then: Should serialize correctly with special characters
        assert!(json.contains("auth0|user@example.com"));
    }

    #[test]
    fn claims_with_long_sub_id() {
        // Given: Claims with a very long subject ID
        let long_id = "a".repeat(1000);
        let claims = Claims {
            sub: long_id.clone(),
            _permissions: None,
        };

        // When: Creating response
        let response = MeResponse { id: claims.sub };

        // Then: Should preserve the full ID
        assert_eq!(response.id.len(), 1000);
        assert_eq!(response.id, long_id);
    }

    #[test]
    fn claims_with_unicode_in_sub() {
        // Given: Claims with Unicode characters in subject
        let claims = Claims {
            sub: "ユーザー|test".to_string(),
            _permissions: None,
        };

        // When: Serializing and deserializing
        let response = MeResponse { id: claims.sub };
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: MeResponse = serde_json::from_str(&json).unwrap();

        // Then: Should preserve Unicode characters
        assert_eq!(deserialized.id, "ユーザー|test");
    }

    // ============================================
    // Handler Function Tests
    // ============================================

    #[tokio::test]
    async fn me_handler_returns_user_id_from_claims() {
        // Given: Claims with a subject
        let claims = Claims {
            sub: "auth0|123".to_string(),
            _permissions: None,
        };
        let state = make_test_state();

        // When: Calling me_handler
        let response = me_handler(claims, state).await;

        // Then: Response ID should match claims subject
        assert_eq!(response.0.id, "auth0|123");
    }

    #[tokio::test]
    async fn me_handler_with_permissions_returns_user_id() {
        // Given: Claims with permissions
        let mut permissions = HashSet::new();
        permissions.insert("read:books".to_string());

        let claims = Claims {
            sub: "auth0|admin456".to_string(),
            _permissions: Some(permissions),
        };
        let state = make_test_state();

        // When: Calling me_handler
        let response = me_handler(claims, state).await;

        // Then: Response ID should match claims subject (permissions are not exposed)
        assert_eq!(response.0.id, "auth0|admin456");
    }

    // ============================================
    // make_test_state helper tests
    // ============================================

    #[test]
    fn make_test_state_has_correct_audience() {
        // Given / When: Creating a test state
        let state = make_test_state();

        // Then: JWT audience should match the test fixture value
        assert_eq!(state.0.jwt_config.audience, "test");
    }

    #[test]
    fn make_test_state_has_correct_domain() {
        // Given / When: Creating a test state
        let state = make_test_state();

        // Then: JWT domain should match the test fixture value
        assert_eq!(state.0.jwt_config.domain, "test-issuer.local");
    }

    #[tokio::test]
    async fn make_test_state_cache_is_initially_empty() {
        // Given / When: A freshly created test state
        let state = make_test_state();

        // Then: The JWKS cache should contain no entries
        assert_eq!(state.0.jwks_cache.entry_count(), 0);
    }

    #[tokio::test]
    async fn make_test_state_creates_independent_cache_instances() {
        // Given: Two independently created test states
        let state_a = make_test_state();
        let state_b = make_test_state();

        // Populate the cache of state_a by inserting an entry
        use jsonwebtoken::jwk::JwkSet;
        let empty_jwks = Arc::new(JwkSet { keys: vec![] });
        state_a.0.jwks_cache.insert((), empty_jwks).await;

        // Then: state_a should have the entry
        assert!(
            state_a.0.jwks_cache.get(&()).await.is_some(),
            "state_a cache should have the inserted entry"
        );
        // And state_b's cache must not be affected (they are separate instances)
        assert!(
            state_b.0.jwks_cache.get(&()).await.is_none(),
            "Caches from make_test_state() must be independent"
        );
    }

    #[tokio::test]
    async fn me_handler_response_is_json_serializable() {
        // Given: A handler response
        let claims = Claims {
            sub: "auth0|serialize_test".to_string(),
            _permissions: None,
        };
        let state = make_test_state();

        // When: Calling me_handler and serializing the inner value
        let response = me_handler(claims, state).await;
        let json = serde_json::to_string(&response.0).unwrap();

        // Then: The JSON should contain the user ID
        assert!(json.contains("auth0|serialize_test"));
        assert!(json.contains("\"id\""));
    }
}