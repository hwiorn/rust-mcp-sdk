# Cloudflare Worker MCP Server - Deployment Guide

This guide covers deploying the Rust/WASM MCP server to Cloudflare Workers.

## Prerequisites Checklist

Run `make check` to verify all prerequisites:

```bash
$ make check
✓ Rust installed
✓ WASM target
✓ wasm-pack
✓ wrangler
```

If any are missing, install them:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install wasm-pack
cargo install wasm-pack

# Install wrangler
npm install -g wrangler
```

## Step-by-Step Deployment

### 1. Configure Cloudflare Account

First time setup:
```bash
wrangler login
```

This opens a browser for authentication.

### 2. Build the WASM Module

```bash
make build
```

This runs `wasm-pack build --target web --out-dir pkg --no-opt` and creates:
- `pkg/*.wasm` - The WASM binary
- `pkg/*.js` - JavaScript bindings
- `pkg/*.d.ts` - TypeScript definitions

### 3. Test Locally

```bash
make dev
```

Opens http://localhost:8787 - test with:
```bash
make test-local
```

### 4. Deploy to Production

```bash
make deploy
```

Your server will be available at:
```
https://<worker-name>.<subdomain>.workers.dev
```

### 5. Verify Deployment

```bash
make test-prod
```

This runs comprehensive tests against the production URL.

## Configuration Files

### wrangler.toml
```toml
name = "mcp-sdk-worker"              # Worker name (appears in URL)
main = "worker-rust.js"              # Entry point (WASM wrapper)
compatibility_date = "2024-11-05"    # Cloudflare API version

[dev]
port = 8787                          # Local dev port
```

### Cargo.toml
```toml
[package.metadata.wasm-pack]
wasm-opt = false                     # Disable to avoid bulk memory errors

[dependencies]
worker = "0.4"                       # Cloudflare Workers SDK
pmcp = { path = "../..", features = ["wasm"] }

[lib]
crate-type = ["cdylib"]              # Required for WASM

[profile.release]
lto = true                           # Link-time optimization
strip = true                         # Strip debug symbols
codegen-units = 1                    # Single codegen unit
opt-level = "z"                      # Size optimization
```

## Deployment Options

### Custom Domain

Add to wrangler.toml:
```toml
routes = [
  { pattern = "mcp.yourdomain.com/*", custom_domain = true }
]
```

### Environment Variables

Add secrets:
```bash
wrangler secret put API_KEY
```

Access in code:
```rust
env.secret("API_KEY")?.to_string()
```

### KV Storage

Create namespace:
```bash
wrangler kv:namespace create "MCP_DATA"
```

Add to wrangler.toml:
```toml
[[kv_namespaces]]
binding = "MCP_DATA"
id = "your-namespace-id"
```

## Monitoring & Debugging

### View Logs
```bash
make logs
# or
wrangler tail mcp-sdk-worker
```

### Check Status
```bash
curl https://your-worker.workers.dev
```

### Debug Build
For readable JavaScript output:
```bash
make build-debug
```

## Troubleshooting

### Issue: Build Fails with Bulk Memory Error

**Error:** `Bulk memory operations require bulk memory`

**Solution:** Ensure `wasm-opt = false` in Cargo.toml or use:
```bash
wasm-pack build --target web --out-dir pkg --no-opt
```

### Issue: Runtime Error "(void 0) is not a function"

**Cause:** Using `worker-build` instead of `wasm-pack`

**Solution:** Use the provided Makefile which uses `wasm-pack` and the JavaScript wrapper.

### Issue: 400 Bad Request from MCP Clients

**Cause:** Missing `capabilities` field in initialize request

**Solution:** The server automatically adds an empty `capabilities` object if missing.

### Issue: CORS Errors in Browser

**Solution:** The server includes CORS headers:
```rust
headers.set("Access-Control-Allow-Origin", "*")?;
headers.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS")?;
headers.set("Access-Control-Allow-Headers", "Content-Type")?;
```

## Performance Optimization

### Bundle Size
Current: ~566KB WASM

Optimizations applied:
- LTO (Link Time Optimization)
- Strip debug symbols
- Single codegen unit
- Size optimization level

### Cold Start Time
Typical: 50-200ms

Tips:
- Keep WASM size small
- Minimize dependencies
- Use `--no-opt` flag (wasm-opt can increase size)

### Request Limits
- Max request size: 100MB
- Max response size: 25MB
- Timeout: 30 seconds (free tier)

## Rollback

If deployment fails:
```bash
# List deployments
wrangler deployments list

# Rollback to previous
wrangler rollback [deployment-id]
```

## CI/CD Integration

### GitHub Actions
```yaml
name: Deploy to Cloudflare

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: wasm32-unknown-unknown

      - name: Install wasm-pack
        run: cargo install wasm-pack

      - name: Build WASM
        run: make build

      - name: Deploy
        uses: cloudflare/wrangler-action@v3
        with:
          apiToken: ${{ secrets.CF_API_TOKEN }}
```

## Security Best Practices

1. **Use Secrets for Sensitive Data**
   ```bash
   wrangler secret put API_KEY
   ```

2. **Validate All Input**
   ```rust
   let params = serde_json::from_value(params)
       .map_err(|_| Response::error("Invalid params", 400))?;
   ```

3. **Rate Limiting**
   Use Cloudflare's built-in rate limiting in dashboard

4. **Authentication**
   Add bearer token validation:
   ```rust
   let auth = req.headers().get("Authorization")?;
   if !auth.starts_with("Bearer ") {
       return Response::error("Unauthorized", 401);
   }
   ```

## Cost Optimization

### Free Tier Limits
- 100,000 requests/day
- 10ms CPU time/invocation

### Monitoring Usage
```bash
# View metrics
wrangler tail --format json | jq '.outcome'
```

### Tips
- Cache responses when possible
- Minimize CPU-intensive operations
- Use KV for data storage instead of computation

## Support Resources

- [Cloudflare Workers Docs](https://developers.cloudflare.com/workers/)
- [workers-rs Documentation](https://docs.rs/worker/)
- [wasm-pack Guide](https://rustwasm.github.io/wasm-pack/)
- [MCP Protocol Spec](https://modelcontextprotocol.io/)

## Quick Reference

```bash
# Build and deploy
make build && make deploy

# View logs
make logs

# Run tests
make test-prod

# Clean and rebuild
make clean && make build

# Check deployment status
curl https://your-worker.workers.dev
```