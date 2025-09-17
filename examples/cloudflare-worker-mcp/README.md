# MCP Server on Cloudflare Workers

This example demonstrates deploying an MCP (Model Context Protocol) server to Cloudflare Workers, showcasing the new transport-agnostic architecture of the Rust MCP SDK.

## Features

- 🚀 **Edge Deployment**: Runs at Cloudflare's edge locations worldwide
- ⚡ **Zero Cold Starts**: Workers stay warm, providing consistent low latency
- 🔧 **Multiple Tools**: Weather, Calculator, and KV Storage tools
- 🌍 **Global Scale**: Automatically scales to handle millions of requests
- 💰 **Cost Effective**: Free tier includes 100,000 requests/day

## Architecture

```
┌─────────────────┐       ┌─────────────────────┐       ┌──────────────┐
│   MCP Client    │──────▶│ Cloudflare Worker   │──────▶│  MCP Server  │
│  (LLM/Claude)   │ HTTP  │   (Edge Runtime)    │       │   (Rust)     │
└─────────────────┘       └─────────────────────┘       └──────────────┘
                                     │
                                     ▼
                          ┌─────────────────────┐
                          │   Tools Available   │
                          ├─────────────────────┤
                          │ • Weather API       │
                          │ • Calculator        │
                          │ • KV Storage        │
                          └─────────────────────┘
```

## Prerequisites

1. **Rust** (latest stable)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Cloudflare Account** (free tier works)
   - Sign up at https://dash.cloudflare.com/sign-up

3. **Wrangler CLI** (Cloudflare's deployment tool)
   ```bash
   npm install -g wrangler
   # Note: Do NOT use `cargo install wrangler` - that's an old deprecated version
   ```

4. **wasm32 target**
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

## Setup

1. **Authenticate with Cloudflare**
   ```bash
   wrangler login
   ```

2. **Configure your account ID** (optional)
   ```bash
   # Find your account ID
   wrangler whoami
   
   # Update wrangler.toml with your account_id
   ```

3. **Install worker-build**
   ```bash
   cargo install worker-build
   ```

## Quick Start

```bash
# Install prerequisites
make install

# Check everything is ready
make check

# Run development server
make dev

# Run tests
make test

# Deploy to Cloudflare
make deploy
```

## Local Development

1. **Run locally with development server**
   ```bash
   make dev
   ```
   
   The server will be available at `http://localhost:8787`

2. **Test with the included client**
   ```bash
   # In another terminal
   make test-client
   ```

3. **Run integration tests**
   ```bash
   make integration-test
   ```

## Deployment

### Deploy to staging (workers.dev subdomain)

```bash
make deploy
```

Your server will be available at:
`https://mcp-server.<your-subdomain>.workers.dev`

### Deploy to production

```bash
make production
```

### Deploy to custom domain

1. Add your domain to Cloudflare
2. Update `wrangler.toml`:
   ```toml
   route = "https://your-domain.com/mcp/*"
   ```
3. Deploy:
   ```bash
   make production
   ```

## Makefile Targets

```bash
make help          # Show all available targets
make install       # Install prerequisites
make check         # Verify setup
make build         # Build WASM binary
make dev           # Run local dev server
make test          # Run tests
make test-client   # Test against local server
make deploy        # Deploy to staging
make production    # Deploy to production
make tail          # Stream live logs
make logs          # View recent logs
make clean         # Clean build artifacts
make quality-gate  # Run formatting and linting checks
make ci            # Run full CI pipeline
```

## Testing the Deployed Server

### Using curl

```bash
# Initialize connection
curl -X POST https://mcp-server.<your-subdomain>.workers.dev \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2024-11-05",
      "capabilities": {},
      "clientInfo": {
        "name": "test-client",
        "version": "1.0.0"
      }
    }
  }'

# List available tools
curl -X POST https://mcp-server.<your-subdomain>.workers.dev \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/list",
    "params": {}
  }'

# Call the calculator tool
curl -X POST https://mcp-server.<your-subdomain>.workers.dev \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "calculator",
      "arguments": {
        "operation": "multiply",
        "a": 42,
        "b": 3.14
      }
    }
  }'
```

### Using the test client

```bash
# Build and run the test client
cargo build --bin test-client
./target/debug/test-client https://mcp-server.<your-subdomain>.workers.dev
```

## Available Tools

### 1. Weather Tool
Get weather information for a location.

**Parameters:**
- `location` (string): City or location name

**Example:**
```json
{
  "name": "weather",
  "arguments": {
    "location": "San Francisco"
  }
}
```

### 2. Calculator Tool
Perform mathematical operations.

**Parameters:**
- `operation` (string): "add", "subtract", "multiply", or "divide"
- `a` (number): First operand
- `b` (number): Second operand

**Example:**
```json
{
  "name": "calculator",
  "arguments": {
    "operation": "multiply",
    "a": 10,
    "b": 5
  }
}
```

### 3. KV Storage Tool
Key-value storage operations.

**Parameters:**
- `action` (string): "get", "set", or "delete"
- `key` (string): Storage key
- `value` (string): Value (required for "set" action)

**Example:**
```json
{
  "name": "kv_storage",
  "arguments": {
    "action": "set",
    "key": "user_preference",
    "value": "dark_mode"
  }
}
```

## Performance Optimization

1. **Bundle Size**: The example is optimized for size with:
   - `opt-level = "z"` - Optimize for smallest binary
   - `lto = true` - Link-time optimization
   - `codegen-units = 1` - Better optimization

2. **Cold Starts**: Cloudflare Workers have virtually no cold starts

3. **Global Distribution**: Workers run at 200+ edge locations

## Monitoring

View logs and metrics in the Cloudflare dashboard:
```bash
# Stream logs
wrangler tail

# View in dashboard
open https://dash.cloudflare.com
```

## Cost

Cloudflare Workers pricing (as of 2024):
- **Free Tier**: 100,000 requests/day
- **Paid**: $5/month for 10 million requests
- **No charges for**: Bandwidth, storage, or DNS queries

## Advanced Features

### Enable KV Storage

1. Create a KV namespace:
   ```bash
   wrangler kv:namespace create "MCP_STORE"
   ```

2. Update `wrangler.toml` with the namespace ID

3. Modify the KVStorageTool to use actual Cloudflare KV

### Add Authentication

Implement authentication in the worker:
```rust
// Check for API key in headers
let api_key = req.headers().get("X-API-Key")?;
if api_key != env.var("EXPECTED_API_KEY")?.to_string() {
    return Response::error("Unauthorized", 401);
}
```

### Rate Limiting

Already configured in `wrangler.toml`:
- 100 requests per minute per IP
- Customize as needed

## Troubleshooting

### Common Issues

1. **WASM size too large**
   - Enable more aggressive optimization
   - Remove unused dependencies
   - Consider splitting into multiple workers

2. **Timeout errors**
   - Workers have a 30-second CPU time limit
   - Optimize long-running operations
   - Use Durable Objects for stateful operations

3. **CORS issues**
   - CORS headers are already configured
   - Adjust `Access-Control-Allow-Origin` as needed

### Debug Mode

Enable detailed logging:
```rust
console_log!("Debug: {}", message);
```

View logs:
```bash
wrangler tail --format pretty
```

## Comparison with Other Deployment Options

| Feature | Cloudflare Workers | AWS Lambda@Edge | Vercel Edge | Traditional Server |
|---------|-------------------|-----------------|-------------|-------------------|
| Cold Starts | ❌ None | ✅ Yes | ⚠️ Minimal | ❌ None |
| Global Edge | ✅ 200+ locations | ✅ 13 regions | ✅ Global | ❌ Single region |
| WebAssembly | ✅ Native | ⚠️ Via Node.js | ✅ Native | ❌ Not needed |
| Cost (Free Tier) | 100k req/day | 1M req/month | 100k req/day | ❌ None |
| Max Runtime | 30 sec CPU | 30 sec | 30 sec | ♾️ Unlimited |
| State/Storage | KV, Durable Objects | DynamoDB | KV | Any database |

## Next Steps

1. **Production Deployment**
   - Set up custom domain
   - Configure monitoring and alerts
   - Implement proper error handling
   - Add authentication

2. **Extend Functionality**
   - Add more tools
   - Integrate with external APIs
   - Implement caching strategies
   - Use Durable Objects for sessions

3. **Integration**
   - Connect with Claude or other LLMs
   - Build client applications
   - Create API documentation

## Resources

- [Cloudflare Workers Documentation](https://developers.cloudflare.com/workers/)
- [MCP Specification](https://modelcontextprotocol.org)
- [Rust WASM Book](https://rustwasm.github.io/book/)
- [Worker-rs Documentation](https://github.com/cloudflare/workers-rs)

## License

MIT - See LICENSE file in the root directory