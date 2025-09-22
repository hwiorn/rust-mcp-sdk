# MCP Server Tester

A comprehensive testing tool for Model Context Protocol (MCP) servers, providing protocol compliance validation, capability testing, and diagnostic features.

## Features

- **Protocol Compliance Testing**: Validates JSON-RPC 2.0 and MCP protocol compliance
- **Multi-Transport Support**: Tests HTTP, HTTPS, WebSocket, and stdio transports
- **Comprehensive Diagnostics**: Layer-by-layer connection troubleshooting
- **Server Comparison**: Compare capabilities and performance between servers
- **Tool Testing**: Discover and test individual tools with custom arguments
- **Scenario Testing**: Define and run complex test scenarios from YAML/JSON files
- **Assertion Framework**: Validate server responses with powerful assertions
- **Multiple Output Formats**: Pretty, JSON, minimal, and verbose outputs
- **CI/CD Ready**: JSON output for automated testing pipelines

## Installation

```bash
# From the examples directory
cd examples/26-server-tester
cargo build --release

# The binary will be at target/release/mcp-tester
```

## Usage

### Quick Start

```bash
# Test a local HTTP server
mcp-tester test http://localhost:8080

# Test with tools validation
mcp-tester test http://localhost:8080 --with-tools

# Test a stdio server
mcp-tester test stdio

# Quick connectivity check
mcp-tester quick http://localhost:8080

# Test OAuth-protected server with access token
mcp-tester test https://api.example.com/mcp --api-key YOUR_ACCESS_TOKEN
```

### Testing OAuth-Protected MCP Servers

For servers that require OAuth authentication:

1. **Obtain an access token** from the MCP Inspector or your OAuth provider
2. **Use the `--api-key` parameter** to pass the token:
   ```bash
   mcp-tester test https://your-oauth-server.com/mcp --api-key "YOUR_ACCESS_TOKEN"
   ```
3. The tester will automatically add the `Authorization: Bearer YOUR_ACCESS_TOKEN` header to all requests

**Note**: You can also set the token via environment variable:
```bash
export MCP_API_KEY="YOUR_ACCESS_TOKEN"
mcp-tester test https://your-oauth-server.com/mcp
```

### Commands

#### `test` - Run Full Test Suite

```bash
mcp-tester test <URL> [OPTIONS]

Options:
  --with-tools           Test all discovered tools
  --tool <NAME>          Test specific tool
  --args <JSON>          Tool arguments as JSON
  --format <FORMAT>      Output format (pretty|json|minimal|verbose)
  --timeout <SECONDS>    Connection timeout (default: 30)
  --insecure            Skip TLS certificate verification
```

#### `compliance` - Protocol Compliance Validation

```bash
mcp-tester compliance <URL> [OPTIONS]

Options:
  --strict              Treat warnings as failures
```

#### `tools` - Discover and Test Tools

```bash
mcp-tester tools <URL> [OPTIONS]

Options:
  --test-all            Test each tool with sample data
```

#### `diagnose` - Connection Diagnostics

```bash
mcp-tester diagnose <URL> [OPTIONS]

Options:
  --network             Include network-level diagnostics
```

#### `compare` - Compare Two Servers

```bash
mcp-tester compare <SERVER1> <SERVER2> [OPTIONS]

Options:
  --with-perf           Include performance comparison
```

#### `health` - Server Health Check

```bash
mcp-tester health <URL>
```

#### `scenario` - Run Test Scenarios

```bash
mcp-tester scenario <URL> <SCENARIO_FILE> [OPTIONS]

Options:
  --detailed       Show detailed step-by-step output

Examples:
# Run a basic test scenario
mcp-tester scenario http://localhost:8080 scenarios/basic-test.yaml

# Run with detailed output
mcp-tester scenario http://localhost:8080 scenarios/complex-workflow.json --detailed

# Run performance tests
mcp-tester scenario http://localhost:8080 scenarios/performance-test.yaml
```

Test scenarios allow you to define complex test sequences with variables, assertions, and workflows. See [SCENARIO_FORMAT.md](SCENARIO_FORMAT.md) for detailed documentation on creating test scenarios.

## Examples

### Testing an OAuth-enabled Server

```bash
# Test the OAuth example server
cd ../25-oauth-basic
make run-http &  # Start server in background

# Run tests
mcp-tester test http://localhost:8080 --with-tools

# Test specific tool with arguments
mcp-tester test http://localhost:8080 \
  --tool admin_action \
  --args '{"action": "test"}'

# Test AI-evals MCP server with OAuth token
# First, get the access token from MCP Inspector after OAuth login
mcp-tester test https://9nq2m33mi0.execute-api.us-west-2.amazonaws.com/mcp \
  --api-key "YOUR_ACCESS_TOKEN_FROM_MCP_INSPECTOR"

# Test with tools
mcp-tester test https://9nq2m33mi0.execute-api.us-west-2.amazonaws.com/mcp \
  --api-key "YOUR_ACCESS_TOKEN" \
  --with-tools
```

### CI/CD Integration

```bash
# Output JSON for automated testing
mcp-tester test $SERVER_URL --format json > test-results.json

# Check exit code
if [ $? -eq 0 ]; then
  echo "All tests passed"
else
  echo "Tests failed"
  exit 1
fi
```

### Debugging Connection Issues

```bash
# Run comprehensive diagnostics
mcp-tester diagnose http://localhost:8080 --network

# This will test:
# - URL validation
# - DNS resolution
# - TCP connectivity
# - TLS/SSL certificates (for HTTPS)
# - HTTP response
# - MCP protocol handshake
```

### Running Test Scenarios

```bash
# Run a basic test scenario
mcp-tester scenario http://localhost:8080 scenarios/basic-test.yaml

# Create a custom scenario for your server
cat > my-test.yaml << EOF
name: My Server Test
steps:
  - name: List available tools
    operation:
      type: list_tools
    assertions:
      - type: success
      - type: array_length
        path: tools
        greater_than: 0
        
  - name: Test my custom tool
    operation:
      type: tool_call
      tool: my_tool
      arguments:
        param: "test"
    assertions:
      - type: success
      - type: exists
        path: result
EOF

# Run the custom scenario
mcp-tester scenario http://localhost:8080 my-test.yaml --verbose
```

### Comparing Server Implementations

```bash
# Compare two servers
mcp-tester compare http://server1.example.com http://server2.example.com --with-perf

# This compares:
# - Protocol versions
# - Capabilities
# - Available tools
# - Performance metrics
```

## Output Formats

### Pretty (Default)

Color-coded terminal output with symbols:
- ✓ Passed (green)
- ✗ Failed (red)  
- ⚠ Warning (yellow)
- ○ Skipped (gray)

### JSON

Structured output for programmatic processing:

```json
{
  "tests": [
    {
      "name": "Initialize",
      "category": "Core",
      "status": "Passed",
      "duration": { "secs": 0, "nanos": 123456 },
      "error": null,
      "details": "Server: my-server v1.0.0, Protocol: 2024-11-05"
    }
  ],
  "summary": {
    "total": 10,
    "passed": 8,
    "failed": 1,
    "warnings": 1,
    "skipped": 0
  },
  "duration": { "secs": 2, "nanos": 500000000 }
}
```

### Minimal

Single-line summary:
```
PASS: 8 passed, 0 failed, 1 warnings in 2.50s
```

### Verbose

Detailed output with full error messages and protocol exchanges.

## Test Categories

### Core Tests
- Connection establishment
- Server initialization
- Capability discovery
- Health endpoints

### Protocol Tests
- JSON-RPC 2.0 compliance
- MCP protocol version validation
- Required method implementation
- Error code standards

### Tool Tests
- Tool discovery (tools/list)
- Tool invocation
- Input validation
- Response format validation

### Performance Tests
- Connection latency
- Response times
- Throughput measurements

## Deployment-Specific Testing

### AWS Lambda

```bash
# Test Lambda function via API Gateway
mcp-tester test https://abc123.execute-api.us-east-1.amazonaws.com/prod

# Lambda cold starts may timeout - increase timeout
mcp-tester test <LAMBDA_URL> --timeout 60
```

### Docker Containers

```bash
# Test containerized server
docker run -p 8080:8080 my-mcp-server

mcp-tester test http://localhost:8080
```

### Kubernetes

```bash
# Port-forward to test in-cluster service
kubectl port-forward service/mcp-server 8080:80

mcp-tester test http://localhost:8080
```

## Troubleshooting

### Connection Refused

```bash
# Run diagnostics to identify the issue
mcp-tester diagnose http://localhost:8080 --network

# Common solutions:
# - Verify server is running
# - Check port is correct
# - Review firewall settings
```

### TLS Certificate Errors

```bash
# For self-signed certificates
mcp-tester test https://localhost:8443 --insecure
```

### Protocol Version Mismatch

The tester supports protocol versions:
- 2024-11-05 (current)
- 2025-03-26
- 2025-06-18

### Timeout Issues

```bash
# Increase timeout for slow servers
mcp-tester test <URL> --timeout 120
```

## Integration with CI/CD

### GitHub Actions

```yaml
- name: Test MCP Server
  run: |
    cargo run --bin mcp-tester -- test ${{ env.SERVER_URL }} --format json > results.json
    
- name: Upload Test Results
  uses: actions/upload-artifact@v2
  with:
    name: test-results
    path: results.json
```

### Jenkins

```groovy
stage('Test MCP Server') {
  steps {
    sh 'mcp-tester test ${SERVER_URL} --format json > results.json'
    archiveArtifacts 'results.json'
  }
}
```

## Contributing

The server tester is part of the Rust MCP SDK. Contributions are welcome!

### Adding New Tests

1. Add test logic to `src/tester.rs`
2. Add validators to `src/validators.rs`
3. Update test categories in `src/report.rs`
4. Add CLI options in `src/main.rs`

### Testing the Tester

```bash
# Run against known good server
cargo run -- test http://localhost:8080

# Run against test fixtures
cargo test
```

## License

MIT - See LICENSE file in the repository root.