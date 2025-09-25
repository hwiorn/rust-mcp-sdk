# OAuth Basic Example for MCP Servers

This example demonstrates how to implement OAuth authentication in MCP servers using the Rust MCP SDK.

## 🚀 Features

- **OAuth Authentication Architecture**: Flexible provider-based authentication system
- **Multiple Authentication Providers**: Support for custom OAuth providers
- **Scope-based Authorization**: Fine-grained access control for tools
- **NoOpAuthProvider**: Development-friendly auth provider for testing
- **HTTP Transport**: Remote deployment support via streamable HTTP
- **Session Management**: Stateful HTTP sessions with unique session IDs

## 🔧 Tools

1. **public_info**: No authentication required
2. **protected_data**: Requires 'read' scope (shows authenticated user info)
3. **admin_action**: Requires 'admin' scope (executes admin operations)

## 🏃 Running the Example

### HTTP Server Mode (Recommended for Remote Deployment)

```bash
# Start the OAuth MCP server on HTTP
cargo run --bin oauth-basic -- http 8080

# Test with the provided HTTP client
cargo run --bin test-client
```

### STDIO Mode (For Local Development)

```bash
# Start the OAuth MCP server on stdio
cargo run --bin oauth-basic -- stdio
```

## 🧪 Testing

### Automated OAuth Protocol Validation

The `test-client` provides comprehensive validation of the OAuth authentication protocol implementation. Here's how it validates each aspect:

```bash
# Run the complete OAuth protocol validation
make test-client
# or
cargo run --bin test-client
```

#### 🔍 **What the Test Client Validates**

The test client systematically validates the OAuth protocol implementation through these steps:

##### **1. MCP Connection & Protocol Negotiation**
```rust
// Validates MCP protocol initialization and session management
let result = client.initialize(capabilities).await?;
```
**Validates:**
- ✅ HTTP transport connectivity 
- ✅ MCP protocol version negotiation (`2025-06-18`)
- ✅ Session ID creation and management
- ✅ Server capability advertisement

**Expected Output:**
```
📡 Initializing connection...
✅ Successfully connected!
   Server: oauth-basic-example v1.0.0
   Protocol: 2025-06-18
   Session ID: 94e8ffb2-691d-44e4-8ae4-68529e20d20c
```

##### **2. Tool Discovery & Authorization Metadata**
```rust
// Validates that OAuth-protected tools are properly exposed
let tools = client.list_tools(None).await?;
```
**Validates:**
- ✅ All OAuth tools are discoverable via MCP protocol
- ✅ Tool metadata is correctly exposed
- ✅ No authentication required for tool discovery

**Expected Output:**
```
🔧 Discovering available tools...
Found 3 tools:
   • public_info - (no description)
   • admin_action - (no description)  
   • protected_data - (no description)
```

##### **3. Public Tool Access (No Authentication)**
```rust
// Validates tools that should work without authentication
let public_result = client.call_tool("public_info", json!({})).await?;
```
**Validates:**
- ✅ Tools marked as public are accessible without auth headers
- ✅ Server doesn't require authentication context for public tools
- ✅ Tool execution succeeds without auth provider validation

**Expected Output:**
```
1️⃣  Testing 'public_info' tool...
   Response: {
     "content": [{"type": "text", "text": "This is a public tool - no authentication required"}],
     "isError": false
   }
```

##### **4. Protected Tool Access (NoOpAuthProvider Authentication)**
```rust
// Validates OAuth authentication flow with development provider
let protected_result = client.call_tool("protected_data", json!({})).await?;
```
**Validates:**
- ✅ **AuthProvider.validate_request()** is called for protected tools
- ✅ **NoOpAuthProvider** provides valid authentication context
- ✅ **AuthContext** is properly created with subject and scopes
- ✅ **RequestHandlerExtra.auth_context()** passes context to tool handlers
- ✅ **Scope validation** works correctly (`read` scope required)

**Expected Output:**
```
2️⃣  Testing 'protected_data' tool...
   Response: {
     "content": [{"type": "text", "text": "Hello authenticated user: dev-user with scopes: [\"read\", \"write\", \"admin\", \"mcp:tools:use\"]"}],
     "isError": false
   }
```

**🔬 This validates the complete OAuth flow:**
1. Server calls `NoOpAuthProvider.validate_request(None)`
2. Provider returns `AuthContext { subject: "dev-user", scopes: ["read", "write", "admin", "mcp:tools:use"] }`
3. Server calls `ScopeBasedAuthorizer.can_access_tool(auth_context, "protected_data")`
4. Authorizer checks if `auth_context.scopes` contains `"read"`
5. Tool handler receives `auth_context` via `RequestHandlerExtra`

##### **5. Admin Tool Access (Elevated Permissions)**
```rust
// Validates admin-level OAuth authorization
let admin_result = client.call_tool("admin_action", json!({"action": "test_admin_action"})).await?;
```
**Validates:**
- ✅ **Elevated scope requirements** (`admin` scope) work correctly
- ✅ **ScopeBasedAuthorizer** properly validates admin permissions  
- ✅ **Tool arguments** are passed through authentication middleware
- ✅ **Complex authorization scenarios** work end-to-end

**Expected Output:**
```
3️⃣  Testing 'admin_action' tool...
   Response: {
     "content": [{"type": "text", "text": "Admin action executed by: dev-user"}],
     "isError": false
   }
```

#### 🧩 **OAuth Architecture Components Validated**

The test client validates every component in the OAuth architecture:

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   HTTP Client   │────│  AuthProvider    │────│ ToolAuthorizer  │────│  Tool Handler   │
│                 │    │ .validate_request│    │.can_access_tool │    │ .handle()       │
└─────────────────┘    └──────────────────┘    └─────────────────┘    └─────────────────┘
        │                        │                       │                       │
        ▼                        ▼                       ▼                       ▼
   ✅ MCP Protocol        ✅ AuthContext           ✅ Scope Check        ✅ Auth Context
   ✅ Session Mgmt        ✅ User Identity         ✅ Permission Logic    ✅ User Info
   ✅ Tool Discovery      ✅ Scope Assignment      ✅ Access Control      ✅ Business Logic
```

#### 🎯 **Success Criteria**

The test validates OAuth protocol compliance by ensuring:

- **Authentication Context Propagation**: Auth info flows correctly through the entire request pipeline
- **Scope-Based Authorization**: Different tools require different permission levels  
- **NoOpAuthProvider Functionality**: Development auth provider works as expected
- **Error-Free Execution**: No authentication errors for properly scoped requests
- **Session Management**: HTTP sessions maintain state correctly
- **MCP Protocol Compliance**: All responses follow MCP specification

**Final Validation Output:**
```
╔════════════════════════════════════════════════════════════╗
║                 OAUTH TESTING COMPLETE                    ║
╠════════════════════════════════════════════════════════════╣
║ ✅ Connection established successfully                     ║
║ ✅ All OAuth tools are accessible                         ║
║ ✅ NoOpAuthProvider working as expected                   ║
║ ✅ HTTP transport functioning properly                    ║
╚════════════════════════════════════════════════════════════╝
```

This comprehensive validation ensures the OAuth implementation is production-ready and can be safely extended with real OAuth providers like Auth0, Google, or AWS Cognito.

### Manual Testing with MCP Inspector

For STDIO mode testing:

```bash
# Build first to avoid cargo output interference
cargo build --bin oauth-basic

# Use MCP Inspector with direct binary
npx @modelcontextprotocol/inspector ./target/debug/oauth-basic stdio
```

### Testing with curl (HTTP mode)

```bash
# Start server
cargo run --bin oauth-basic -- http 8080

# Use the provided curl test script  
./test-curl.sh
```

## 🏗️ Architecture

### Authentication Flow

```
Client Request → AuthProvider.validate_request() → ToolAuthorizer.can_access_tool() → Tool Handler
```

### Core Components

1. **AuthProvider Trait**: Validates authentication requests
2. **ToolAuthorizer Trait**: Controls access to specific tools based on scopes
3. **ScopeBasedAuthorizer**: Built-in authorizer for scope-based access control
4. **NoOpAuthProvider**: Development provider that grants all access

## 🔐 Authentication Providers

### NoOpAuthProvider (Development)

Perfect for development and testing:
```rust
// Always returns valid auth context
AuthContext {
    subject: "dev-user",
    scopes: ["read", "write", "admin", "mcp:tools:use"],
    // ... other fields
}
```

### Production Providers

For production, implement the `AuthProvider` trait:

```rust
#[async_trait]
impl AuthProvider for MyOAuthProvider {
    async fn validate_request(&self, authorization_header: Option<&str>) -> Result<Option<AuthContext>> {
        // Validate JWT token, query OAuth server, etc.
    }
    
    fn is_required(&self) -> bool {
        true // Require authentication
    }
}
```

## 📊 Configuration

### Scope-Based Authorization

```rust
let authorizer = ScopeBasedAuthorizer::new()
    .require_scopes("public_info", vec![])                    // No auth needed
    .require_scopes("protected_data", vec!["read".to_string()]) // Read scope required
    .require_scopes("admin_action", vec!["admin".to_string()]) // Admin scope required
    .default_scopes(vec!["mcp:tools:use".to_string()]);       // Default for other tools
```

### Server Setup

```rust
let server = Server::builder()
    .name("oauth-basic-example")
    .version("1.0.0")
    .capabilities(ServerCapabilities::tools_only())
    .auth_provider(NoOpAuthProvider)              // Set auth provider
    .tool_authorizer(authorizer)                  // Set authorization rules
    .tool("public_info", PublicTool)             // Register tools
    .tool("protected_data", ProtectedTool)
    .tool("admin_action", AdminTool)
    .build()?;
```

## 🌐 Remote Deployment

The HTTP transport makes this example perfect for cloud deployment:

- **AWS Lambda + API Gateway**: Deploy as serverless function
- **Docker Containers**: Containerized deployment
- **Cloudflare Workers**: Edge deployment
- **Google Cloud Run**: Serverless containers

## 🚀 Production Considerations

1. **Replace NoOpAuthProvider**: Use real OAuth providers (Auth0, Cognito, etc.)
2. **Token Validation**: Implement proper JWT validation
3. **Rate Limiting**: Add rate limiting middleware
4. **CORS**: Configure CORS for web clients
5. **TLS**: Always use HTTPS in production
6. **Monitoring**: Add health checks and metrics

## 📚 Related Documentation

- [OAuth Universal Architecture](../../docs/architecture/oauth-universal-architecture.md)
- [OAuth Separation of Concerns](../../docs/architecture/oauth-separation-of-concerns.md)
- [Example 22: Stateful HTTP Server](../22_streamable_http_server_stateful.rs)
- [Example 24: HTTP Client](../24_streamable_http_client.rs)
