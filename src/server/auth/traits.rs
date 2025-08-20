//! Core authentication traits for flexible OAuth/auth integration.

use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Authentication context containing validated user information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    /// Subject identifier (user ID).
    pub subject: String,

    /// Granted scopes/permissions.
    pub scopes: Vec<String>,

    /// Additional claims from the token.
    pub claims: HashMap<String, serde_json::Value>,

    /// Original token if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,

    /// Client ID that authenticated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,

    /// Token expiration timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
}

impl AuthContext {
    /// Check if the context has a specific scope.
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().any(|s| s == scope)
    }

    /// Check if the context has all specified scopes.
    pub fn has_all_scopes(&self, scopes: &[&str]) -> bool {
        scopes.iter().all(|scope| self.has_scope(scope))
    }

    /// Check if the context has any of the specified scopes.
    pub fn has_any_scope(&self, scopes: &[&str]) -> bool {
        scopes.iter().any(|scope| self.has_scope(scope))
    }

    /// Check if the token is expired.
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            expires_at < now
        } else {
            false
        }
    }
}

/// Core authentication provider trait.
/// This is the main abstraction that MCP servers use for authentication.
#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// Validate an incoming request and extract authentication context.
    ///
    /// This method receives the authorization header value and should:
    /// 1. Parse the authentication token (e.g., Bearer token)
    /// 2. Validate the token
    /// 3. Return the authentication context if valid
    ///
    /// The `authorization_header` parameter contains the value of the Authorization header,
    /// if present (e.g., "Bearer eyJhbGci...")
    async fn validate_request(
        &self,
        authorization_header: Option<&str>,
    ) -> Result<Option<AuthContext>>;

    /// Get the authentication scheme this provider uses (e.g., "Bearer", "Basic").
    fn auth_scheme(&self) -> &'static str {
        "Bearer"
    }

    /// Check if this provider requires authentication for all requests.
    fn is_required(&self) -> bool {
        true
    }
}

/// Token validator trait for validating access tokens.
#[async_trait]
pub trait TokenValidator: Send + Sync {
    /// Validate an access token and return token information.
    async fn validate(&self, token: &str) -> Result<AuthContext>;

    /// Optionally validate token with additional context (e.g., required scopes).
    async fn validate_with_context(
        &self,
        token: &str,
        required_scopes: Option<&[&str]>,
    ) -> Result<AuthContext> {
        let auth_context = self.validate(token).await?;

        // Check required scopes if specified
        if let Some(scopes) = required_scopes {
            if !auth_context.has_all_scopes(scopes) {
                return Err(crate::error::Error::protocol(
                    crate::error::ErrorCode::INVALID_REQUEST,
                    "Insufficient scopes",
                ));
            }
        }

        Ok(auth_context)
    }
}

/// Session management trait for stateful authentication.
#[async_trait]
pub trait SessionManager: Send + Sync {
    /// Create a new session and return the session ID.
    async fn create_session(&self, auth: AuthContext) -> Result<String>;

    /// Get session by ID.
    async fn get_session(&self, session_id: &str) -> Result<Option<AuthContext>>;

    /// Update an existing session.
    async fn update_session(&self, session_id: &str, auth: AuthContext) -> Result<()>;

    /// Invalidate a session.
    async fn invalidate_session(&self, session_id: &str) -> Result<()>;

    /// Clean up expired sessions (optional background task).
    async fn cleanup_expired(&self) -> Result<usize> {
        Ok(0) // Default no-op implementation
    }
}

/// Tool authorization trait for fine-grained access control.
#[async_trait]
pub trait ToolAuthorizer: Send + Sync {
    /// Check if the authenticated context can access a specific tool.
    async fn can_access_tool(&self, auth: &AuthContext, tool_name: &str) -> Result<bool>;

    /// Get required scopes for a tool.
    async fn required_scopes_for_tool(&self, tool_name: &str) -> Result<Vec<String>>;
}

/// Simple scope-based tool authorizer.
#[derive(Debug, Clone)]
pub struct ScopeBasedAuthorizer {
    tool_scopes: HashMap<String, Vec<String>>,
    default_scopes: Vec<String>,
}

impl ScopeBasedAuthorizer {
    /// Create a new scope-based authorizer.
    pub fn new() -> Self {
        Self {
            tool_scopes: HashMap::new(),
            default_scopes: vec!["mcp:tools:use".to_string()],
        }
    }

    /// Add required scopes for a tool.
    pub fn require_scopes(mut self, tool_name: impl Into<String>, scopes: Vec<String>) -> Self {
        self.tool_scopes.insert(tool_name.into(), scopes);
        self
    }

    /// Set default required scopes for all tools.
    pub fn default_scopes(mut self, scopes: Vec<String>) -> Self {
        self.default_scopes = scopes;
        self
    }
}

#[async_trait]
impl ToolAuthorizer for ScopeBasedAuthorizer {
    async fn can_access_tool(&self, auth: &AuthContext, tool_name: &str) -> Result<bool> {
        let required_scopes = self
            .tool_scopes
            .get(tool_name)
            .unwrap_or(&self.default_scopes);

        let scope_refs: Vec<&str> = required_scopes.iter().map(|s| s.as_str()).collect();
        Ok(auth.has_all_scopes(&scope_refs))
    }

    async fn required_scopes_for_tool(&self, tool_name: &str) -> Result<Vec<String>> {
        Ok(self
            .tool_scopes
            .get(tool_name)
            .unwrap_or(&self.default_scopes)
            .clone())
    }
}

impl Default for ScopeBasedAuthorizer {
    fn default() -> Self {
        Self::new()
    }
}
