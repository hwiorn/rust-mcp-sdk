# OAuth Separation of Concerns for Rust MCP SDK

## Executive Summary

This document defines the boundaries between what OAuth functionality belongs in the Rust MCP SDK versus application repositories, based on analysis of existing implementations and the principle of maintaining clean separation of concerns.

## Core Principles

1. **SDK provides interfaces, not implementations** - The SDK defines traits and minimal implementations
2. **Applications own business logic** - OAuth provider specifics, scope requirements, and authorization rules
3. **Infrastructure remains separate** - DCR proxies, authorizers, and platform-specific code live in dedicated services
4. **Flexibility over opinions** - The SDK enables patterns without forcing architectural decisions

## What Belongs in the Rust MCP SDK

### Core OAuth Abstractions

```rust
// Core traits that define the OAuth contract
pub mod auth {
    pub mod traits {
        #[async_trait]
        pub trait AuthProvider: Send + Sync {
            async fn validate_request(&self, headers: &HeaderMap) -> Result<AuthContext>;
        }
        
        #[async_trait]
        pub trait TokenValidator: Send + Sync {
            async fn validate(&self, token: &str) -> Result<TokenInfo>;
        }
        
        #[async_trait]
        pub trait SessionManager: Send + Sync {
            async fn create_session(&self, auth: AuthContext) -> Result<SessionId>;
            async fn get_session(&self, id: &SessionId) -> Result<Option<AuthContext>>;
            async fn invalidate_session(&self, id: &SessionId) -> Result<()>;
        }
    }
}
```

### Basic Implementations

```rust
pub mod providers {
    // ProxyProvider - delegates to upstream OAuth server (like TypeScript SDK)
    pub struct ProxyProvider {
        upstream_url: String,
        verify_token: Box<dyn Fn(&str) -> Future<Output = Result<AuthContext>>>,
    }
    
    // NoOpProvider - for development/testing
    pub struct NoOpProvider;
    
    // JwtValidator - generic JWT validation with JWKS support
    pub struct JwtValidator {
        jwks_url: String,
        issuer: String,
        audience: String,
    }
    
    // InMemorySessionManager - simple session storage for development
    pub struct InMemorySessionManager {
        sessions: Arc<DashMap<String, (AuthContext, Instant)>>,
    }
}
```

### Integration Points

```rust
// Server builder extensions
impl ServerBuilder {
    pub fn auth_provider(mut self, provider: impl AuthProvider + 'static) -> Self;
    pub fn protect_tool(mut self, tool_name: &str, required_scope: &str) -> Self;
}

// Middleware for OAuth integration
pub struct OAuthMiddleware {
    provider: Arc<dyn AuthProvider>,
}

// Transport extensions
pub trait OAuthTransport {
    fn with_auth(&mut self, token: &str) -> &mut Self;
}
```

### Utilities

- PKCE support utilities
- OIDC discovery helpers
- Token extraction from headers
- Basic JWT parsing (without provider-specific validation)

## What Belongs in Application Repositories

### Business Logic and Tools

```rust
// examples/mcp-todos/src/tools/
mod todo_tools {
    pub struct ListTodosTool { /* business logic */ }
    pub struct CreateTodoTool { /* business logic */ }
    pub struct UpdateTodoTool { /* business logic */ }
}

// Application-specific authorization rules
mod auth_rules {
    pub fn required_scopes_for_tool(tool: &str) -> Vec<String> {
        match tool {
            "list_todos" => vec!["todos:read"],
            "create_todo" => vec!["todos:write"],
            _ => vec![]
        }
    }
}
```

### Provider-Specific Implementations

```rust
// Cognito adapter with DCR proxy integration
mod cognito {
    pub struct CognitoProvider {
        dcr_proxy_url: String,  // URL to separate DCR proxy Lambda
        user_pool_id: String,
        region: String,
    }
    
    impl AuthProvider for CognitoProvider {
        // Cognito-specific implementation
    }
}

// Google OAuth implementation
mod google {
    pub struct GoogleProvider {
        client_id: String,
        client_secret: String,
    }
}
```

### Deployment Configurations

```rust
// Lambda-specific wrapper
mod lambda_deployment {
    pub async fn handle_mcp_request(
        event: LambdaEvent,
        server: Arc<Server>
    ) -> Result<Response> {
        // Lambda + API Gateway specific handling
    }
}

// Docker configuration
mod docker_deployment {
    pub fn configure_for_docker() -> ServerConfig {
        // Docker-specific settings
    }
}
```

## What Belongs in Separate Infrastructure Services

### DCR Proxy Service (e.g., rust-oauth2-proxy)
- Standalone Lambda/service for Dynamic Client Registration
- Works around provider limitations (e.g., Cognito's lack of DCR)
- Not part of the MCP server itself

### Token Authorizer (e.g., rust-authorizer)
- API Gateway Lambda authorizer
- Platform-specific token validation
- JWKS caching and rotation
- Can be shared across multiple MCP servers

### Infrastructure as Code
- Terraform/CDK definitions
- API Gateway configurations
- Lambda function definitions
- Environment-specific settings

## Lambda Deployment Architecture Options

### Option 1: Three Separate Lambdas (Current Pattern)

```
┌──────────────────┐     ┌──────────────────┐     ┌──────────────────┐
│   DCR Proxy      │     │  Token Authorizer │     │   MCP Server     │
│   Lambda         │     │     Lambda        │     │     Lambda       │
│                  │     │                   │     │                  │
│ - Client reg     │     │ - Token validate  │     │ - Business logic │
│ - PKCE support   │     │ - JWKS cache      │     │ - Tools          │
│                  │     │ - Claims extract  │     │ - Routing        │
└──────────────────┘     └──────────────────┘     └──────────────────┘
        ↑                         ↑                         ↑
        │                         │                         │
        └─────────────────────────┴─────────────────────────┘
                          API Gateway
```

**Benefits:**
- Clear separation of concerns
- Independent scaling
- Reusable authorizer across services
- Minimal cold start for each function
- DCR proxy can serve multiple MCP servers

**Drawbacks:**
- More infrastructure to manage
- Additional network hops
- Complex deployment

### Option 2: Monolithic Lambda (New Design Enables)

```
┌────────────────────────────────────┐
│        MCP Server Lambda            │
│                                     │
│  ┌──────────────────────────────┐  │
│  │   ProxyProvider (SDK)        │  │
│  │   - Delegates to DCR proxy   │  │
│  └──────────────────────────────┘  │
│                                     │
│  ┌──────────────────────────────┐  │
│  │   Token Validation           │  │
│  │   - Inline validation        │  │
│  └──────────────────────────────┘  │
│                                     │
│  ┌──────────────────────────────┐  │
│  │   MCP Tools & Routing        │  │
│  │   - Business logic           │  │
│  └──────────────────────────────┘  │
└────────────────────────────────────┘
                 ↑
                 │
           API Gateway
```

**Benefits:**
- Simpler deployment
- Single cold start
- No inter-Lambda communication
- Easier local testing

**Drawbacks:**
- Larger Lambda package
- Can't reuse authorizer
- All-or-nothing scaling

### Option 3: Hybrid Approach (Recommended)

```
┌──────────────────┐     ┌────────────────────────────────────┐
│   DCR Proxy      │     │      MCP Server Lambda              │
│   Lambda         │     │                                     │
│                  │←────│  ┌──────────────────────────────┐  │
│ - Shared service │     │  │  ProxyProvider (SDK)         │  │
│ - Multi-tenant   │     │  └──────────────────────────────┘  │
└──────────────────┘     │                                     │
                         │  ┌──────────────────────────────┐  │
   API Gateway           │  │  Inline Token Validation     │  │
   Authorizer            │  │  - Fast path validation      │  │
   (Optional)            │  └──────────────────────────────┘  │
        ↓                │                                     │
        └────────────────│  ┌──────────────────────────────┐  │
                         │  │  MCP Tools & Business Logic  │  │
                         │  └──────────────────────────────┘  │
                         └────────────────────────────────────┘
```

**Benefits:**
- DCR proxy remains shared infrastructure
- Token validation can be inline for performance
- Optional API Gateway authorizer for additional security
- Clean architecture with SDK abstractions
- Flexible deployment options

**The new SDK design enables all three patterns** - applications can choose based on their needs.

## Lambda/API Gateway/Cognito Example Recommendation

### Should it be an SDK example?

**Yes, but as an advanced example** with clear structure:

```
examples/
├── 28-oauth-lambda-cognito/
│   ├── README.md                    # Clear prerequisites and setup
│   ├── mcp-server/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs              # MCP server with ProxyProvider
│   │   │   ├── cognito_adapter.rs   # Cognito-specific auth
│   │   │   └── lambda_handler.rs    # Lambda integration
│   │   └── Dockerfile
│   ├── infrastructure/
│   │   ├── dcr-proxy/               # Separate DCR proxy (optional)
│   │   │   ├── Cargo.toml
│   │   │   └── src/main.rs
│   │   ├── terraform/               # IaC for AWS resources
│   │   │   ├── api_gateway.tf
│   │   │   ├── lambda.tf
│   │   │   └── cognito.tf
│   │   └── scripts/
│   │       ├── deploy.sh
│   │       └── test_auth.sh
│   └── docs/
│       ├── architecture.md
│       └── setup_guide.md
```

### Example Implementation

```rust
// examples/28-oauth-lambda-cognito/mcp-server/src/main.rs
use pmcp::{Server, ServerBuilder};
use pmcp::auth::{ProxyProvider, AuthContext};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Configuration from environment
    let dcr_proxy_url = env::var("DCR_PROXY_URL")?;
    let inline_validation = env::var("INLINE_TOKEN_VALIDATION")
        .map(|v| v == "true")
        .unwrap_or(false);
    
    // Create OAuth provider
    let auth_provider = if inline_validation {
        // Inline validation (monolithic approach)
        Box::new(CognitoProvider::new(
            env::var("USER_POOL_ID")?,
            env::var("REGION")?
        ))
    } else {
        // Delegate to DCR proxy (hybrid approach)
        Box::new(ProxyProvider::new()
            .upstream_url(&dcr_proxy_url)
            .token_validator(Box::new(validate_cognito_token)))
    };
    
    // Build MCP server with auth
    let server = Server::builder()
        .name("cognito-protected-mcp")
        .auth_provider(auth_provider)
        .tool("protected_tool", ProtectedTool::new())
        .protect_tool("protected_tool", "mcp:tools:use")
        .build()?;
    
    // Run as Lambda
    let server = Arc::new(server);
    run(service_fn(move |event: LambdaEvent<ApiGatewayProxyRequest>| {
        let server = server.clone();
        async move {
            handle_lambda_request(event, server).await
        }
    })).await
}

async fn validate_cognito_token(token: &str) -> Result<AuthContext> {
    // Token validation logic
    // Can use API Gateway authorizer context if available
    // Or perform inline validation
}
```

### Documentation Structure

The example should include:

1. **Prerequisites Guide**
   - AWS account setup
   - Cognito user pool creation
   - API Gateway configuration

2. **Architecture Decisions**
   - When to use monolithic vs separated Lambdas
   - Performance considerations
   - Security trade-offs

3. **Deployment Options**
   - Terraform for full automation
   - Manual setup instructions
   - Testing procedures

4. **Migration Path**
   - From existing three-Lambda setup
   - To simplified architecture

## Benefits of This Design

1. **Flexibility**: SDK doesn't force architectural decisions
2. **Clean boundaries**: Clear separation between SDK, app, and infrastructure
3. **Migration friendly**: Existing three-Lambda setups continue to work
4. **Performance options**: Choose between separated or monolithic based on needs
5. **Reusability**: DCR proxy and authorizers can be shared across services
6. **Testing**: Each component can be tested independently

## Conclusion

The SDK should provide the minimal OAuth abstractions needed to enable secure MCP servers while letting applications and infrastructure repositories handle the specifics. This approach maintains clean separation of concerns while enabling both simple and complex deployment patterns.

The Lambda/API Gateway/Cognito example should be included as an advanced example (example 28) that demonstrates the flexibility of the SDK's auth abstractions. It should show both the separated (three Lambda) and monolithic approaches, letting users choose based on their requirements.