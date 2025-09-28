//! Enhanced typed tools with optional output typing
//!
//! Provides TypedToolV2 with support for both input and output type schemas.
//! Output schemas are not part of the MCP protocol but useful for testing and documentation.

use crate::{Error, RequestHandlerExtra, Result, ToolHandler, ToolInfo};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::fmt;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;

#[cfg(feature = "schema-generation")]
use schemars::JsonSchema;

/// A typed tool with both input and output type safety
pub struct TypedToolV2<TIn, TOut, F>
where
    TIn: DeserializeOwned + Send + Sync + 'static,
    TOut: Serialize + Send + Sync + 'static,
    F: Fn(TIn, RequestHandlerExtra) -> Pin<Box<dyn Future<Output = Result<TOut>> + Send>>
        + Send
        + Sync,
{
    name: String,
    description: Option<String>,
    input_schema: Value,
    output_schema: Option<Value>,
    handler: F,
    _phantom: PhantomData<(TIn, TOut)>,
}

impl<TIn, TOut, F> fmt::Debug for TypedToolV2<TIn, TOut, F>
where
    TIn: DeserializeOwned + Send + Sync + 'static,
    TOut: Serialize + Send + Sync + 'static,
    F: Fn(TIn, RequestHandlerExtra) -> Pin<Box<dyn Future<Output = Result<TOut>> + Send>>
        + Send
        + Sync,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypedToolV2")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("input_schema", &self.input_schema)
            .field("output_schema", &self.output_schema)
            .finish()
    }
}

impl<TIn, TOut, F> TypedToolV2<TIn, TOut, F>
where
    TIn: DeserializeOwned + Send + Sync + 'static,
    TOut: Serialize + Send + Sync + 'static,
    F: Fn(TIn, RequestHandlerExtra) -> Pin<Box<dyn Future<Output = Result<TOut>> + Send>>
        + Send
        + Sync,
{
    /// Create a new typed tool with automatic schema generation
    #[cfg(feature = "schema-generation")]
    pub fn new(name: impl Into<String>, handler: F) -> Self
    where
        TIn: JsonSchema,
        TOut: JsonSchema,
    {
        let input_schema = generate_schema::<TIn>();
        let output_schema = Some(generate_schema::<TOut>());

        Self {
            name: name.into(),
            description: None,
            input_schema,
            output_schema,
            handler,
            _phantom: PhantomData,
        }
    }

    /// Create without output schema generation
    #[cfg(feature = "schema-generation")]
    pub fn new_input_only(name: impl Into<String>, handler: F) -> Self
    where
        TIn: JsonSchema,
    {
        let input_schema = generate_schema::<TIn>();

        Self {
            name: name.into(),
            description: None,
            input_schema,
            output_schema: None,
            handler,
            _phantom: PhantomData,
        }
    }

    /// Create with manually provided schemas
    pub fn new_with_schemas(
        name: impl Into<String>,
        input_schema: Value,
        output_schema: Option<Value>,
        handler: F,
    ) -> Self {
        Self {
            name: name.into(),
            description: None,
            input_schema,
            output_schema,
            handler,
            _phantom: PhantomData,
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Get the output schema (for testing/documentation)
    pub fn output_schema(&self) -> Option<&Value> {
        self.output_schema.as_ref()
    }

    /// Validate output against the schema (for testing)
    /// Note: This is a simplified validation for demonstration.
    /// For production, use the validation feature with jsonschema crate.
    pub fn validate_output(&self, output: &TOut) -> Result<()> {
        // Basic check that output can be serialized
        let _output_json = serde_json::to_value(output)
            .map_err(|e| Error::Internal(format!("Failed to serialize output: {}", e)))?;

        // In debug mode, log schema information
        #[cfg(debug_assertions)]
        if let Some(_schema) = &self.output_schema {
            tracing::debug!(
                "Output schema available for tool '{}' - use validation feature for full validation",
                self.name
            );
        }

        Ok(())
    }
}

#[async_trait]
impl<TIn, TOut, F> ToolHandler for TypedToolV2<TIn, TOut, F>
where
    TIn: DeserializeOwned + Send + Sync + 'static,
    TOut: Serialize + Send + Sync + 'static,
    F: Fn(TIn, RequestHandlerExtra) -> Pin<Box<dyn Future<Output = Result<TOut>> + Send>>
        + Send
        + Sync,
{
    async fn handle(&self, args: Value, extra: RequestHandlerExtra) -> Result<Value> {
        // Deserialize and validate input
        let typed_args: TIn = serde_json::from_value(args).map_err(|e| {
            Error::Validation(format!("Invalid arguments for tool '{}': {}", self.name, e))
        })?;

        // Call the handler
        let result = (self.handler)(typed_args, extra).await?;

        // Optionally validate output in debug mode
        #[cfg(debug_assertions)]
        {
            if let Err(e) = self.validate_output(&result) {
                tracing::warn!("Tool '{}' output validation failed: {}", self.name, e);
            }
        }

        // Serialize the result
        serde_json::to_value(result)
            .map_err(|e| Error::Internal(format!("Failed to serialize tool output: {}", e)))
    }

    fn metadata(&self) -> Option<ToolInfo> {
        Some(ToolInfo {
            name: self.name.clone(),
            description: self.description.clone(),
            input_schema: self.input_schema.clone(),
        })
    }
}

/// Synchronous version with input and output typing
pub struct TypedSyncToolV2<TIn, TOut, F>
where
    TIn: DeserializeOwned + Send + Sync + 'static,
    TOut: Serialize + Send + Sync + 'static,
    F: Fn(TIn, RequestHandlerExtra) -> Result<TOut> + Send + Sync,
{
    name: String,
    description: Option<String>,
    input_schema: Value,
    output_schema: Option<Value>,
    handler: F,
    _phantom: PhantomData<(TIn, TOut)>,
}

impl<TIn, TOut, F> fmt::Debug for TypedSyncToolV2<TIn, TOut, F>
where
    TIn: DeserializeOwned + Send + Sync + 'static,
    TOut: Serialize + Send + Sync + 'static,
    F: Fn(TIn, RequestHandlerExtra) -> Result<TOut> + Send + Sync,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypedSyncToolV2")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("input_schema", &self.input_schema)
            .field("output_schema", &self.output_schema)
            .finish()
    }
}

impl<TIn, TOut, F> TypedSyncToolV2<TIn, TOut, F>
where
    TIn: DeserializeOwned + Send + Sync + 'static,
    TOut: Serialize + Send + Sync + 'static,
    F: Fn(TIn, RequestHandlerExtra) -> Result<TOut> + Send + Sync,
{
    /// Create with automatic schema generation
    #[cfg(feature = "schema-generation")]
    pub fn new(name: impl Into<String>, handler: F) -> Self
    where
        TIn: JsonSchema,
        TOut: JsonSchema,
    {
        let input_schema = generate_schema::<TIn>();
        let output_schema = Some(generate_schema::<TOut>());

        Self {
            name: name.into(),
            description: None,
            input_schema,
            output_schema,
            handler,
            _phantom: PhantomData,
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Get the output schema
    pub fn output_schema(&self) -> Option<&Value> {
        self.output_schema.as_ref()
    }
}

#[async_trait]
impl<TIn, TOut, F> ToolHandler for TypedSyncToolV2<TIn, TOut, F>
where
    TIn: DeserializeOwned + Send + Sync + 'static,
    TOut: Serialize + Send + Sync + 'static,
    F: Fn(TIn, RequestHandlerExtra) -> Result<TOut> + Send + Sync,
{
    async fn handle(&self, args: Value, extra: RequestHandlerExtra) -> Result<Value> {
        // Deserialize input
        let typed_args: TIn = serde_json::from_value(args).map_err(|e| {
            Error::Validation(format!("Invalid arguments for tool '{}': {}", self.name, e))
        })?;

        // Call the handler
        let result = (self.handler)(typed_args, extra)?;

        // Serialize output
        serde_json::to_value(result)
            .map_err(|e| Error::Internal(format!("Failed to serialize tool output: {}", e)))
    }

    fn metadata(&self) -> Option<ToolInfo> {
        Some(ToolInfo {
            name: self.name.clone(),
            description: self.description.clone(),
            input_schema: self.input_schema.clone(),
        })
    }
}

/// Generate a JSON schema for a type
#[cfg(feature = "schema-generation")]
fn generate_schema<T: JsonSchema>() -> Value {
    let schema = schemars::schema_for!(T);
    let json_schema = serde_json::to_value(&schema).unwrap_or_else(|_| {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "additionalProperties": true
        })
    });

    // Normalize the schema
    crate::server::schema_utils::normalize_schema(json_schema)
}

// Builder methods for ServerBuilder to use V2 tools
// NOTE: Uncomment when using with validation feature
// impl crate::ServerBuilder {
//     /// Add a typed tool with input and output schemas
//     #[cfg(feature = "schema-generation")]
//     pub fn tool_typed_v2<TIn, TOut, F, Fut>(
//         mut self,
//         name: impl Into<String>,
//         handler: F,
//     ) -> Self
//     where
//         TIn: DeserializeOwned + JsonSchema + Send + Sync + 'static,
//         TOut: Serialize + JsonSchema + Send + Sync + 'static,
//         F: Fn(TIn, crate::RequestHandlerExtra) -> Fut + Send + Sync + 'static,
//         Fut: Future<Output = Result<TOut>> + Send + 'static,
//     {
//         use std::sync::Arc;

//         let name_str = name.into();
//         let wrapped_handler = move |args: TIn, extra: crate::RequestHandlerExtra| -> Pin<Box<dyn Future<Output = Result<TOut>> + Send>> {
//             Box::pin(handler(args, extra))
//         };

//         let tool = TypedToolV2::new(name_str.clone(), wrapped_handler);
//         self.tools.insert(name_str, Arc::new(tool));
//         self
//     }
// }
