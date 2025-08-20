//! Server-side authentication providers and middleware.

pub mod middleware;
pub mod oauth2;
pub mod proxy;
pub mod traits;

// Re-export core traits
pub use traits::{
    AuthContext, AuthProvider, ScopeBasedAuthorizer, SessionManager, TokenValidator, ToolAuthorizer,
};

// Re-export proxy providers
pub use proxy::{NoOpAuthProvider, OptionalAuthProvider, ProxyProvider, ProxyProviderConfig};

// Keep existing OAuth2 exports for compatibility
pub use oauth2::{
    AccessToken, AuthorizationCode, AuthorizationRequest, GrantType, InMemoryOAuthProvider,
    OAuthClient, OAuthError, OAuthMetadata, OAuthProvider, ProxyOAuthProvider, ResponseType,
    RevocationRequest, TokenInfo, TokenRequest, TokenType,
};

// Note: AuthContext from traits replaces the one from middleware
// We'll need to update middleware to use the new AuthContext from traits
