use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use std::time::Duration;
use tracing_subscriber;

mod diagnostics;
mod report;
mod tester;
mod validators;

use report::{OutputFormat, TestReport};
use tester::ServerTester;

#[derive(Parser)]
#[command(name = "mcp-tester")]
#[command(about = "Comprehensive MCP server testing tool")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format
    #[arg(short, long, global = true, default_value = "pretty")]
    format: OutputFormat,

    /// Verbosity level (0-3)
    #[arg(short, long, global = true, default_value = "0")]
    verbose: u8,

    /// Connection timeout in seconds
    #[arg(long, global = true, default_value = "30")]
    timeout: u64,

    /// Skip TLS certificate verification (insecure)
    #[arg(long, global = true)]
    insecure: bool,

    /// API key for authentication (if required)
    #[arg(long, global = true, env = "MCP_API_KEY")]
    api_key: Option<String>,

    /// Force specific transport type (http|stdio)
    #[arg(long, global = true)]
    transport: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run full test suite
    Test {
        /// Server URL (http://localhost:8080 or stdio)
        url: String,

        /// Test specific tools
        #[arg(long)]
        with_tools: bool,

        /// Specific tool to test
        #[arg(long)]
        tool: Option<String>,

        /// Tool arguments as JSON
        #[arg(long)]
        args: Option<String>,
    },

    /// Quick connectivity check
    Quick {
        /// Server URL
        url: String,
    },

    /// Protocol compliance validation
    Compliance {
        /// Server URL
        url: String,

        /// Strict mode (fail on warnings)
        #[arg(long)]
        strict: bool,
    },

    /// List and test available tools
    Tools {
        /// Server URL
        url: String,

        /// Test each tool with sample data
        #[arg(long)]
        test_all: bool,
    },

    /// Connection diagnostics
    Diagnose {
        /// Server URL
        url: String,

        /// Include network diagnostics
        #[arg(long)]
        network: bool,
    },

    /// Compare two servers
    Compare {
        /// First server URL
        server1: String,

        /// Second server URL
        server2: String,

        /// Include performance comparison
        #[arg(long)]
        with_perf: bool,
    },

    /// Server health check
    Health {
        /// Server URL
        url: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging - respect RUST_LOG if set, otherwise use verbosity
    let env_filter = if std::env::var("RUST_LOG").is_ok() {
        tracing_subscriber::EnvFilter::from_default_env()
    } else {
        let log_level = match cli.verbose {
            0 => "error",
            1 => "warn",
            2 => "info",
            3 => "debug",
            _ => "trace",
        };
        tracing_subscriber::EnvFilter::new(format!(
            "mcp_server_tester={},pmcp={}",
            log_level, log_level
        ))
    };

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .init();

    // Print header
    if matches!(cli.format, OutputFormat::Pretty) {
        print_header();
    }

    // Execute command
    let result = match cli.command {
        Commands::Test {
            url,
            with_tools,
            tool,
            args,
        } => run_full_test(&url, with_tools, tool, args, cli.timeout, cli.insecure, cli.api_key.as_deref(), cli.transport.as_deref()).await,

        Commands::Quick { url } => run_quick_test(&url, cli.timeout, cli.insecure, cli.api_key.as_deref(), cli.transport.as_deref()).await,

        Commands::Compliance { url, strict } => {
            run_compliance_test(&url, strict, cli.timeout, cli.insecure, cli.api_key.as_deref(), cli.transport.as_deref()).await
        },

        Commands::Tools { url, test_all } => {
            run_tools_test(&url, test_all, cli.timeout, cli.insecure, cli.api_key.as_deref(), cli.transport.as_deref()).await
        },

        Commands::Diagnose { url, network } => {
            run_diagnostics(&url, network, cli.timeout, cli.insecure, cli.api_key.as_deref()).await
        },

        Commands::Compare {
            server1,
            server2,
            with_perf,
        } => run_comparison(&server1, &server2, with_perf, cli.timeout, cli.insecure, cli.api_key.as_deref(), cli.transport.as_deref()).await,

        Commands::Health { url } => run_health_check(&url, cli.timeout, cli.insecure, cli.api_key.as_deref(), cli.transport.as_deref()).await,
    };

    // Handle results and output
    match result {
        Ok(report) => {
            report.print(cli.format);
            if report.has_failures() && cli.format == OutputFormat::Pretty {
                std::process::exit(1);
            }
        },
        Err(e) => {
            if cli.format == OutputFormat::Json {
                let error_report = TestReport::from_error(e);
                error_report.print(OutputFormat::Json);
            } else {
                eprintln!("{} {:#}", "Error:".red().bold(), e);
            }
            std::process::exit(1);
        },
    }

    Ok(())
}

fn print_header() {
    println!(
        "{}",
        "╔════════════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "║              MCP SERVER TESTING TOOL v0.1.0               ║"
            .cyan()
            .bold()
    );
    println!(
        "{}",
        "╚════════════════════════════════════════════════════════════╝".cyan()
    );
    println!();
}

async fn run_full_test(
    url: &str,
    with_tools: bool,
    tool: Option<String>,
    args: Option<String>,
    timeout: u64,
    insecure: bool,
    api_key: Option<&str>,
    transport: Option<&str>,
) -> Result<TestReport> {
    let mut tester = ServerTester::new(url, Duration::from_secs(timeout), insecure, api_key, transport)?;

    println!("{}", "Running full test suite...".green());
    println!();

    // Run all test categories
    let mut report = tester.run_full_suite(with_tools).await?;

    // Test specific tool if requested
    if let Some(tool_name) = tool {
        println!("{} {}", "Testing specific tool:".yellow(), tool_name);
        let tool_args = if let Some(args_str) = args {
            serde_json::from_str(&args_str).context("Invalid JSON arguments")?
        } else {
            serde_json::Value::Null
        };

        let tool_result = tester.test_tool(&tool_name, tool_args).await?;
        report.add_test(tool_result);
    }

    Ok(report)
}

async fn run_quick_test(url: &str, timeout: u64, insecure: bool, api_key: Option<&str>, transport: Option<&str>) -> Result<TestReport> {
    let mut tester = ServerTester::new(url, Duration::from_secs(timeout), insecure, api_key, transport)?;

    println!("{}", "Running quick connectivity test...".green());
    println!();

    tester.run_quick_test().await
}

async fn run_compliance_test(
    url: &str,
    strict: bool,
    timeout: u64,
    insecure: bool,
    api_key: Option<&str>,
    transport: Option<&str>,
) -> Result<TestReport> {
    let mut tester = ServerTester::new(url, Duration::from_secs(timeout), insecure, api_key, transport)?;

    println!("{}", "Running protocol compliance tests...".green());
    if strict {
        println!("{}", "Mode: STRICT (warnings will fail)".yellow());
    }
    println!();

    tester.run_compliance_tests(strict).await
}

async fn run_tools_test(
    url: &str,
    test_all: bool,
    timeout: u64,
    insecure: bool,
    api_key: Option<&str>,
    transport: Option<&str>,
) -> Result<TestReport> {
    let mut tester = ServerTester::new(url, Duration::from_secs(timeout), insecure, api_key, transport)?;

    println!("{}", "Discovering and testing tools...".green());
    println!();

    tester.run_tools_discovery(test_all).await
}

async fn run_diagnostics(
    url: &str,
    network: bool,
    timeout: u64,
    insecure: bool,
    api_key: Option<&str>,
) -> Result<TestReport> {
    println!("{}", "Running connection diagnostics...".green());
    println!();

    let report =
        diagnostics::run_diagnostics(url, network, Duration::from_secs(timeout), insecure, api_key).await?;

    Ok(report)
}

async fn run_comparison(
    server1: &str,
    server2: &str,
    with_perf: bool,
    timeout: u64,
    insecure: bool,
    api_key: Option<&str>,
    transport: Option<&str>,
) -> Result<TestReport> {
    println!("{}", "Comparing servers...".green());
    println!("  Server 1: {}", server1.cyan());
    println!("  Server 2: {}", server2.cyan());
    println!();

    let mut tester1 = ServerTester::new(server1, Duration::from_secs(timeout), insecure, api_key, transport)?;
    let mut tester2 = ServerTester::new(server2, Duration::from_secs(timeout), insecure, api_key, transport)?;

    let report = tester1.compare_with(&mut tester2, with_perf).await?;

    Ok(report)
}

async fn run_health_check(url: &str, timeout: u64, insecure: bool, api_key: Option<&str>, transport: Option<&str>) -> Result<TestReport> {
    let mut tester = ServerTester::new(url, Duration::from_secs(timeout), insecure, api_key, transport)?;

    println!("{}", "Checking server health...".green());
    println!();

    tester.run_health_check().await
}
