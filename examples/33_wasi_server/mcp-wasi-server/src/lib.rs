//! The MCP Server logic compiled to WASM/WASI.
use pmcp::wasi::RequestHandlerWasiRunner;
use pmcp::server::traits::RequestHandler;
use pmcp::types::{CallToolRequest, CallToolResult, ListToolsResult, ToolInfo};
use serde_json::Value;

// Use wit-bindgen to generate the bindings for the WASI HTTP interface.
wit_bindgen::generate!({
    world: "wasi:http/proxy@0.2.0",
});

use crate::wasi::http::types::{Fields, Method, Scheme, OutgoingResponse};
use crate::wasi::http::incoming_handler::{Guest, IncomingRequest, ResponseOutparam};

/// A simple handler that provides a weather tool.
#[derive(Clone)]
struct WeatherToolHandler;

impl pmcp::server::traits::ToolHandler for WeatherToolHandler {
    fn list_tools(&self, _req: pmcp::types::ListToolsRequest) -> pmcp::Result<ListToolsResult> {
        Ok(ListToolsResult {
            tools: vec![ToolInfo {
                name: "get_weather".to_string(),
                description: Some("Gets the current weather for a given city.".to_string()),
                                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "city": {
                            "type": "string",
                            "description": "The city to get the weather for."
                        }
                    },
                    "required": ["city"]
                }),
            }],
            next_cursor: None,
        })
    }

    fn call_tool(&self, req: CallToolRequest) -> pmcp::Result<CallToolResult> {
        if req.name == "get_weather" {
            let city = req.arguments["city"].as_str().unwrap_or("London");
            let url = format!(
                "https://wttr.in/{}?format=j1",
                city
            );

            // Use reqwest to make the outbound HTTP call.
            // This will be proxied by the WASI host.
            let weather_response = futures::executor::block_on(async {
                reqwest::get(&url).await.map_err(|e| pmcp::Error::internal(e.to_string()))?.json::<Value>().await.map_err(|e| pmcp::Error::internal(e.to_string()))
            })?;

            let current_condition = &weather_response["current_condition"][0];
            let temp_c = current_condition["temp_C"].as_str().unwrap();
            let weather_desc = current_condition["weatherDesc"][0]["value"].as_str().unwrap();

            Ok(CallToolResult {
                content: vec![pmcp::types::Content::Text {
                    text: serde_json::to_string(&serde_json::json!({
                        "temperature": format!("{}°C", temp_c),
                        "description": weather_desc,
                    })).unwrap(),
                }],
                is_error: false,
            })
        } else {
            Err(pmcp::Error::not_found(format!("Tool not found: {}", req.name)))
        }
    }
}

impl pmcp::server::traits::PromptHandler for WeatherToolHandler {
    fn handle(
        &self,
        _args: std::collections::HashMap<String, String>,
        _extra: pmcp::shared::cancellation::RequestHandlerExtra,
    ) -> pmcp::Result<pmcp::types::GetPromptResult> {
        Err(pmcp::Error::internal("Prompts are not supported in this example"))
    }
}

impl pmcp::server::traits::ResourceHandler for WeatherToolHandler {
    fn read(
        &self,
        _uri: &str,
        _extra: pmcp::shared::cancellation::RequestHandlerExtra,
    ) -> pmcp::Result<pmcp::types::ReadResourceResult> {
        Err(pmcp::Error::internal("Resources are not supported in this example"))
    }

    fn list(
        &self,
        _cursor: Option<String>,
        _extra: pmcp::shared::cancellation::RequestHandlerExtra,
    ) -> pmcp::Result<pmcp::types::ListResourcesResult> {
        Err(pmcp::Error::internal("Resources are not supported in this example"))
    }
}

impl pmcp::server::traits::SamplingHandler for WeatherToolHandler {
    fn create_message(
        &self,
        _params: pmcp::types::CreateMessageParams,
        _extra: pmcp::shared::cancellation::RequestHandlerExtra,
    ) -> pmcp::Result<pmcp::types::CreateMessageResult> {
        Err(pmcp::Error::internal("Sampling is not supported in this example"))
    }
}

impl pmcp::server::traits::RequestHandler for WeatherToolHandler {}