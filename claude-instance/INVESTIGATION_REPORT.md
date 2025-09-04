# Tool Protection/Authorization Implementation Comparison: TypeScript vs Rust MCP SDKs

## Executive Summary

After investigating both the TypeScript and Rust MCP SDK implementations, I found fundamental differences in how tool authorization and protection are handled. The **TypeScript SDK does not have a builder pattern with tool protection mechanisms**, while the **Rust SDK implements a comprehensive ServerBuilder with built-in tool protection via `protect_tool()` and custom `ToolAuthorizer` support**.

## Key Findings

### TypeScript SDK - No Built-in Tool Protection

The TypeScript MCP SDK (`/Users/guy/Development/mcp/sdk/typescript-sdk`) takes a **middleware-centric approach** but lacks built-in tool protection:

#### Architecture
- **No ServerBuilder Pattern**: Uses direct class instantiation (`new McpServer()`, `new Server()`)
- **No Built-in Tool Authorization**: Tools are registered without authorization checks
- **Express Middleware Approach**: Authorization is handled at the HTTP layer via Express middleware

#### Tool Registration
```typescript
// McpServer class - Direct registration without authorization
server.tool("toolName", callback);
server.registerTool("toolName", config, callback);
```

#### Authorization Implementation
- **Middleware-based**: Uses `requireBearerAuth()` middleware for HTTP endpoints
- **Scope checking**: Bearer token middleware can check required scopes
- **Request-level**: Authorization happens at transport level, not tool level

### Rust SDK - Comprehensive Tool Protection

The Rust MCP SDK (`/Users/guy/Development/mcp/sdk/rust-mcp-sdk`) implements a **builder-centric approach** with extensive tool protection:

#### Architecture
- **ServerBuilder Pattern**: Fluent builder API for server construction
- **Built-in Tool Authorization**: Multiple authorization strategies
- **Compile-time Safety**: Prevents conflicting authorization configurations

#### Tool Protection Mechanisms

##### 1. `protect_tool()` Method
```rust
pub fn protect_tool(mut self, tool_name: impl Into<String>, scopes: Vec<String>) -> Self {
    // Store the tool protection requirements to be applied at build time
    self.tool_protections.insert(tool_name.into(), scopes);
    self
}
```
- Stores tool protections in `HashMap<String, Vec<String>>` during building
- Applied at `build()` time to create a `ScopeBasedAuthorizer`
- Allows protecting individual tools with specific scopes

##### 2. Custom `ToolAuthorizer` Support
```rust
pub fn tool_authorizer(mut self, authorizer: impl auth::ToolAuthorizer + 'static) -> Self {
    if !self.tool_protections.is_empty() {
        eprintln!("Warning: Setting a custom tool_authorizer clears any previous protect_tool() configurations");
        self.tool_protections.clear();
    }
    self.tool_authorizer = Some(Arc::new(authorizer));
    self
}
```
- Supports custom authorization implementations
- Warns when conflicts with `protect_tool()` configurations
- Provides flexibility for complex authorization scenarios

##### 3. ScopeBasedAuthorizer Implementation
```rust
#[derive(Debug, Clone)]
pub struct ScopeBasedAuthorizer {
    tool_scopes: HashMap<String, Vec<String>>,
    default_scopes: Vec<String>,
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
}
```

#### Build-time Validation
```rust
pub fn build(self) -> Result<Server> {
    let tool_authorizer = if !self.tool_protections.is_empty() {
        if self.tool_authorizer.is_some() {
            return Err(crate::Error::validation(
                "Cannot use protect_tool() with a custom tool_authorizer. \
                 Either use protect_tool() to configure scope-based authorization, \
                 or provide a custom ToolAuthorizer implementation, but not both."
            ));
        }
        // Create a ScopeBasedAuthorizer with all the tool protections
        let mut authorizer = auth::ScopeBasedAuthorizer::new();
        for (tool_name, scopes) in self.tool_protections {
            authorizer = authorizer.require_scopes(tool_name, scopes);
        }
        Some(Arc::new(authorizer) as Arc<dyn auth::ToolAuthorizer>)
    } else {
        self.tool_authorizer
    };
    
    // ... rest of server construction
}
```

## Comparative Analysis

### Design Philosophy Differences

| Aspect | TypeScript SDK | Rust SDK |
|--------|----------------|----------|
| **Authorization Level** | Transport/Middleware | Tool-specific |
| **Builder Pattern** | ❌ None | ✅ Comprehensive |
| **Tool Protection** | ❌ Manual middleware setup | ✅ Built-in `protect_tool()` |
| **Configuration Safety** | ❌ Runtime errors possible | ✅ Build-time validation |
| **Flexibility** | ✅ Express middleware ecosystem | ✅ Custom `ToolAuthorizer` trait |

### Authorization Granularity

**TypeScript SDK:**
- **Coarse-grained**: All-or-nothing per endpoint
- **Transport-level**: Authorization happens at HTTP layer
- **Manual setup**: Developers must implement tool-specific checks

**Rust SDK:**
- **Fine-grained**: Per-tool authorization
- **Application-level**: Authorization integrated into tool execution
- **Automatic enforcement**: Built into server request processing

### Error Handling and Validation

**TypeScript SDK:**
- Runtime errors for authorization failures
- No compile-time protection against misconfigurations
- Manual error handling in middleware

**Rust SDK:**
- Build-time validation prevents conflicting configurations
- Type-safe authorization context
- Automatic error responses for unauthorized tool access

## Implementation Examples

### TypeScript - Manual Authorization
```typescript
// No built-in tool protection - must be handled manually
const server = new McpServer(serverInfo);
server.tool("sensitive-tool", async (args, extra) => {
  // Manual authorization check would go here
  // No built-in framework support
  return { result: "data" };
});
```

### Rust - Built-in Protection
```rust
let server = Server::builder()
    .name("secure-server")
    .version("1.0.0")
    .protect_tool("sensitive-tool", vec!["admin".to_string()])
    .tool("sensitive-tool", SensitiveTool)
    .build()?; // Automatic authorization enforcement
```

## Recommendations

### For TypeScript SDK Enhancement
1. **Add ServerBuilder Pattern**: Implement fluent builder API
2. **Built-in Tool Authorization**: Add `protect_tool()` equivalent
3. **Type Safety**: Add TypeScript interfaces for authorization
4. **Integration**: Seamless integration with existing middleware system

### For Rust SDK Maintenance
1. **Documentation**: The current approach is excellent - maintain it
2. **Testing**: Comprehensive test coverage for authorization edge cases
3. **Examples**: More examples showing authorization patterns

## Files Analyzed

### TypeScript SDK Files
- `/Users/guy/Development/mcp/sdk/typescript-sdk/src/server/index.ts`
- `/Users/guy/Development/mcp/sdk/typescript-sdk/src/server/mcp.ts`
- `/Users/guy/Development/mcp/sdk/typescript-sdk/src/server/auth/middleware/bearerAuth.ts`
- `/Users/guy/Development/mcp/sdk/typescript-sdk/src/server/auth/types.ts`

### Rust SDK Files
- `/Users/guy/Development/mcp/sdk/rust-mcp-sdk/src/server/mod.rs` (lines 1051-1580)
- `/Users/guy/Development/mcp/sdk/rust-mcp-sdk/src/server/auth/traits.rs`
- `/Users/guy/Development/mcp/sdk/rust-mcp-sdk/src/server/auth/mod.rs`

## Conclusion

The Rust SDK demonstrates superior tool authorization architecture with:
- **Built-in Protection**: `protect_tool()` method for easy tool security
- **Flexible Authorization**: Custom `ToolAuthorizer` trait implementation
- **Build-time Safety**: Prevents conflicting authorization configurations
- **Automatic Enforcement**: Integrated into server request processing

The TypeScript SDK currently lacks these built-in tool protection mechanisms and relies on manual middleware implementation for authorization.