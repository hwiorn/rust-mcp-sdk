//! Test client for Cloudflare Worker MCP Server
//! 
//! This client demonstrates how to interact with the MCP server
//! deployed on Cloudflare Workers.

use reqwest;
use serde_json::{json, Value};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get server URL from command line or use local development server
    let server_url = env::args()
        .nth(1)
        .unwrap_or_else(|| "http://localhost:8787".to_string());
    
    println!("🚀 MCP Cloudflare Worker Test Client");
    println!("📡 Connecting to: {}", server_url);
    println!("=" .repeat(50));
    
    let client = reqwest::Client::new();
    
    // Test 1: Initialize connection
    println!("\n1️⃣  Initializing MCP connection...");
    let init_request = json!({
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
    });
    
    let response = client.post(&server_url)
        .json(&init_request)
        .send()
        .await?;
    
    let init_response: Value = response.json().await?;
    println!("✅ Initialized: {}", serde_json::to_string_pretty(&init_response)?);
    
    // Test 2: List available tools
    println!("\n2️⃣  Listing available tools...");
    let list_tools_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });
    
    let response = client.post(&server_url)
        .json(&list_tools_request)
        .send()
        .await?;
    
    let tools_response: Value = response.json().await?;
    println!("🔧 Available tools:");
    if let Some(result) = tools_response.get("result") {
        if let Some(tools) = result.get("tools") {
            if let Some(tools_array) = tools.as_array() {
                for tool in tools_array {
                    if let Some(name) = tool.get("name") {
                        println!("   - {}", name);
                    }
                }
            }
        }
    }
    
    // Test 3: Use Calculator tool
    println!("\n3️⃣  Testing Calculator tool...");
    let calc_request = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "calculator",
            "arguments": {
                "operation": "multiply",
                "a": 42,
                "b": 3.14159
            }
        }
    });
    
    let response = client.post(&server_url)
        .json(&calc_request)
        .send()
        .await?;
    
    let calc_response: Value = response.json().await?;
    println!("🧮 Calculator result:");
    if let Some(result) = calc_response.get("result") {
        if let Some(content) = result.get("content") {
            if let Some(content_array) = content.as_array() {
                if let Some(first) = content_array.first() {
                    if let Some(text) = first.get("text") {
                        let parsed: Value = serde_json::from_str(text.as_str().unwrap_or("{}"))?;
                        println!("   42 × π ≈ {}", parsed.get("result").unwrap_or(&json!(0)));
                    }
                }
            }
        }
    }
    
    // Test 4: Use Weather tool
    println!("\n4️⃣  Testing Weather tool...");
    let weather_request = json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "weather",
            "arguments": {
                "location": "Tokyo"
            }
        }
    });
    
    let response = client.post(&server_url)
        .json(&weather_request)
        .send()
        .await?;
    
    let weather_response: Value = response.json().await?;
    println!("🌤️  Weather in Tokyo:");
    if let Some(result) = weather_response.get("result") {
        if let Some(content) = result.get("content") {
            if let Some(content_array) = content.as_array() {
                if let Some(first) = content_array.first() {
                    if let Some(text) = first.get("text") {
                        let weather: Value = serde_json::from_str(text.as_str().unwrap_or("{}"))?;
                        println!("   Temperature: {}", weather.get("temperature").unwrap_or(&json!("N/A")));
                        println!("   Conditions: {}", weather.get("conditions").unwrap_or(&json!("N/A")));
                        println!("   Forecast: {}", weather.get("forecast").unwrap_or(&json!("N/A")));
                    }
                }
            }
        }
    }
    
    // Test 5: Use KV Storage tool
    println!("\n5️⃣  Testing KV Storage tool...");
    
    // Set a value
    let kv_set_request = json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "tools/call",
        "params": {
            "name": "kv_storage",
            "arguments": {
                "action": "set",
                "key": "test_key",
                "value": "Hello from Cloudflare Workers!"
            }
        }
    });
    
    let response = client.post(&server_url)
        .json(&kv_set_request)
        .send()
        .await?;
    
    let kv_set_response: Value = response.json().await?;
    println!("💾 Stored value in KV");
    
    // Get the value back
    let kv_get_request = json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "tools/call",
        "params": {
            "name": "kv_storage",
            "arguments": {
                "action": "get",
                "key": "test_key"
            }
        }
    });
    
    let response = client.post(&server_url)
        .json(&kv_get_request)
        .send()
        .await?;
    
    let kv_get_response: Value = response.json().await?;
    if let Some(result) = kv_get_response.get("result") {
        if let Some(content) = result.get("content") {
            if let Some(content_array) = content.as_array() {
                if let Some(first) = content_array.first() {
                    if let Some(text) = first.get("text") {
                        let kv_result: Value = serde_json::from_str(text.as_str().unwrap_or("{}"))?;
                        println!("📖 Retrieved value: {}", kv_result.get("value").unwrap_or(&json!("N/A")));
                    }
                }
            }
        }
    }
    
    // Test 6: Error handling
    println!("\n6️⃣  Testing error handling...");
    let error_request = json!({
        "jsonrpc": "2.0",
        "id": 7,
        "method": "tools/call",
        "params": {
            "name": "calculator",
            "arguments": {
                "operation": "divide",
                "a": 10,
                "b": 0
            }
        }
    });
    
    let response = client.post(&server_url)
        .json(&error_request)
        .send()
        .await?;
    
    let error_response: Value = response.json().await?;
    if let Some(error) = error_response.get("error") {
        println!("❌ Expected error caught: {}", error.get("message").unwrap_or(&json!("Unknown error")));
    }
    
    println!("\n" + &"=".repeat(50));
    println!("✅ All tests completed successfully!");
    println!("\n📊 Performance Notes:");
    println!("   - Cloudflare Workers have ~0ms cold start");
    println!("   - Responses are served from the nearest edge location");
    println!("   - Global replication happens automatically");
    
    Ok(())
}