# Background Agents with PMCP

This document provides examples of background agent implementations using PMCP as the underlying MCP protocol foundation.

## Overview

Background agents are long-running services that provide continuous functionality through the Model Context Protocol. They run as daemon processes and integrate with AI tools like Claude Code to provide real-time analysis, monitoring, and assistance.

PMCP serves as the transport and protocol foundation that these agents build upon, providing:
- Reliable WebSocket and HTTP transport
- Full MCP protocol compliance  
- High-performance message processing
- Advanced error recovery and connection management

## Example Implementations

### 1. PMAT - Quality Analysis Agent

**Location**: `../paiml-mcp-agent-toolkit`

PMAT (Pragmatic Multi-language Analysis Toolkit) is a comprehensive example of a Claude Code background agent that provides continuous code quality monitoring.

#### Features
- **Real-time Quality Monitoring**: Watches file system changes and provides instant feedback
- **Toyota Way Compliance**: Enforces complexity standards (≤20) with zero technical debt tolerance  
- **AI-Driven Refactoring**: Intelligent suggestions based on code analysis
- **Quality Gate Automation**: Transparent enforcement in development workflows
- **MCP Integration**: Native Claude Code integration via PMCP transport layer

#### Architecture
```
Claude Code → MCP Client → PMAT Agent Server (PMCP) → Quality Analysis Engine
```

#### Quick Start
```bash
# Navigate to PMAT project
cd ../paiml-mcp-agent-toolkit

# Start PMAT as background agent for Claude Code
pmat agent mcp-server

# Or start as daemon for continuous monitoring
pmat agent start --project-path /path/to/project
```

#### Configuration
```json
{
  "mcpServers": {
    "pmat": {
      "command": "pmat",
      "args": ["agent", "mcp-server"],
      "transport": "stdio"
    }
  }
}
```

**Documentation**: See `../paiml-mcp-agent-toolkit/docs/CLAUDE_CODE_AGENT.md`

### 2. Ruchy - Language Server Agent

**Location**: `../ruchy`

Ruchy provides an example of a language-specific background agent that runs as an MCP server for the Ruchy programming language.

#### Features
- **Language Server Protocol**: Full LSP implementation for Ruchy language
- **Background Compilation**: Continuous syntax checking and compilation
- **MCP Integration**: Exposes language features through MCP protocol
- **Self-Hosting**: Demonstrates bootstrapping capabilities

#### Architecture
```
Claude Code → MCP Client → Ruchy Agent Server (PMCP) → Language Analysis
```

#### Quick Start
```bash
# Navigate to Ruchy project
cd ../ruchy

# Start Ruchy MCP server
ruchy mcp-server

# Or use with Claude Code integration
ruchy serve --mode mcp
```

**Documentation**: See `../ruchy/README.md`

## Building Your Own Background Agent

### Using PMCP as Foundation

Here's how to create a background agent using PMCP:

```rust
use pmcp::server::{Server, ServerOptions};
use pmcp::types::{Tool, Resource};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create MCP server using PMCP
    let mut server = Server::new(ServerOptions {
        name: "my-background-agent".to_string(),
        version: "1.0.0".to_string(),
        ..Default::default()
    });

    // Register tools for your agent
    server.add_tool(Tool {
        name: "monitor_project".to_string(),
        description: "Start monitoring a project".to_string(),
        // ... tool implementation
    });

    // Register resources your agent provides
    server.add_resource(Resource {
        uri: "agent://status".to_string(),
        name: "Agent Status".to_string(),
        // ... resource implementation
    });

    // Start the server in background mode
    server.run_stdio().await?;
    
    Ok(())
}
```

### Key Patterns for Background Agents

1. **File System Watching**
   ```rust
   use notify::{Watcher, RecommendedWatcher, RecursiveMode};
   
   // Watch for file changes
   let (tx, rx) = channel();
   let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(1))?;
   watcher.watch("/path/to/project", RecursiveMode::Recursive)?;
   ```

2. **Continuous Processing Loop**
   ```rust
   // Background processing task
   tokio::spawn(async move {
       loop {
           // Process files, analyze, etc.
           analyze_project_state().await;
           tokio::time::sleep(Duration::from_secs(60)).await;
       }
   });
   ```

3. **MCP Tool Integration**
   ```rust
   // Expose agent capabilities as MCP tools
   server.set_tool_handler("analyze_quality", |args| async {
       let analysis = perform_quality_analysis(args).await?;
       Ok(json!(analysis))
   });
   ```

### Configuration Best Practices

Background agents should provide flexible configuration:

```toml
[agent]
name = "my-background-agent"
check_interval_seconds = 60
enable_notifications = true

[monitoring]
watch_patterns = ["**/*.rs", "**/*.py", "**/*.js"]
ignore_patterns = ["target/", "node_modules/", ".git/"]

[quality]
max_complexity = 20
enforce_standards = true
```

## Integration with Claude Code

### MCP Server Configuration

Add your background agent to Claude Code's MCP configuration:

```json
{
  "mcpServers": {
    "my-agent": {
      "command": "my-background-agent",
      "args": ["serve", "--config", "agent.toml"],
      "env": {
        "LOG_LEVEL": "info"
      }
    }
  }
}
```

### Available Commands

Once integrated, your agent tools become available in Claude Code:

- "Start monitoring my project"
- "Check project quality status" 
- "Analyze code complexity"
- "Generate refactoring suggestions"

## Production Deployment

### Systemd Service (Linux)

```ini
[Unit]
Description=My Background Agent
After=network.target

[Service]
Type=simple
User=agent
WorkingDirectory=/opt/my-agent
ExecStart=/usr/local/bin/my-agent serve --config /etc/my-agent/config.toml
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

### Docker Deployment

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/my-agent /usr/local/bin/
COPY config.toml /etc/my-agent/
EXPOSE 8080
CMD ["my-agent", "serve", "--config", "/etc/my-agent/config.toml"]
```

## Performance Considerations

Background agents should be optimized for continuous operation:

- **Memory Management**: Use bounded channels and connection pools
- **CPU Efficiency**: Implement debouncing for file system events  
- **Network Optimization**: Leverage PMCP's connection pooling and compression
- **Error Recovery**: Use PMCP's adaptive retry mechanisms

## Security Best Practices

- **File System Access**: Minimize required permissions
- **Network Security**: Use localhost-only binding for MCP servers
- **Data Privacy**: Keep all processing local, no external data transmission
- **Configuration Security**: Protect sensitive configuration with proper file permissions

## Resources

- **PMAT Documentation**: `../paiml-mcp-agent-toolkit/docs/`
- **Ruchy Documentation**: `../ruchy/README.md` 
- **PMCP API Reference**: [docs.rs/pmcp](https://docs.rs/pmcp)
- **MCP Specification**: [modelcontextprotocol.io](https://modelcontextprotocol.io)

## Contributing

Background agents are an excellent way to showcase PMCP capabilities. If you build an interesting background agent:

1. Create comprehensive documentation
2. Include configuration examples
3. Provide Docker deployment options
4. Add integration tests
5. Submit a PR to add it to this examples list

---

*Background agents represent the future of AI-assisted development - continuous, intelligent, and seamlessly integrated into developer workflows.*