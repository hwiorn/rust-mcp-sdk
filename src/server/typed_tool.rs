//! Type-safe tool implementations with automatic schema generation.

use crate::{types::ToolInfo, Result};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::fmt;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;

use super::cancellation::RequestHandlerExtra;
use super::ToolHandler;

#[cfg(feature = "schema-generation")]
use schemars::JsonSchema;

/// A typed tool implementation with automatic schema generation and validation.
pub struct TypedTool<T, F>
where
    T: DeserializeOwned + Send + Sync + 'static,
    F: Fn(T, RequestHandlerExtra) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>>
        + Send
        + Sync,
{
    name: String,
    description: Option<String>,
    input_schema: Value,
    handler: F,
    _phantom: PhantomData<T>,
}

impl<T, F> fmt::Debug for TypedTool<T, F>
where
    T: DeserializeOwned + Send + Sync + 'static,
    F: Fn(T, RequestHandlerExtra) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>>
        + Send
        + Sync,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypedTool")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("input_schema", &self.input_schema)
            .finish()
    }
}

impl<T, F> TypedTool<T, F>
where
    T: DeserializeOwned + Send + Sync + 'static,
    F: Fn(T, RequestHandlerExtra) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>>
        + Send
        + Sync,
{
    /// Create a new typed tool with automatic schema generation.
    #[cfg(feature = "schema-generation")]
    pub fn new(name: impl Into<String>, handler: F) -> Self
    where
        T: JsonSchema,
    {
        let schema = generate_schema::<T>();
        Self {
            name: name.into(),
            description: None,
            input_schema: schema,
            handler,
            _phantom: PhantomData,
        }
    }

    /// Create a new typed tool with a manually provided schema.
    pub fn new_with_schema(name: impl Into<String>, schema: Value, handler: F) -> Self {
        Self {
            name: name.into(),
            description: None,
            input_schema: schema,
            handler,
            _phantom: PhantomData,
        }
    }

    /// Set the description for this tool.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[async_trait]
impl<T, F> ToolHandler for TypedTool<T, F>
where
    T: DeserializeOwned + Send + Sync + 'static,
    F: Fn(T, RequestHandlerExtra) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>>
        + Send
        + Sync,
{
    async fn handle(&self, args: Value, extra: RequestHandlerExtra) -> Result<Value> {
        // Deserialize and validate the arguments
        let typed_args: T = serde_json::from_value(args).map_err(|e| {
            crate::Error::Validation(format!("Invalid arguments for tool '{}': {}", self.name, e))
        })?;

        // Call the handler with the typed arguments
        (self.handler)(typed_args, extra).await
    }

    fn metadata(&self) -> Option<ToolInfo> {
        Some(ToolInfo {
            name: self.name.clone(),
            description: self.description.clone(),
            input_schema: self.input_schema.clone(),
        })
    }
}

/// A synchronous typed tool implementation with automatic schema generation.
pub struct TypedSyncTool<T, F>
where
    T: DeserializeOwned + Send + Sync + 'static,
    F: Fn(T, RequestHandlerExtra) -> Result<Value> + Send + Sync,
{
    name: String,
    description: Option<String>,
    input_schema: Value,
    handler: F,
    _phantom: PhantomData<T>,
}

impl<T, F> fmt::Debug for TypedSyncTool<T, F>
where
    T: DeserializeOwned + Send + Sync + 'static,
    F: Fn(T, RequestHandlerExtra) -> Result<Value> + Send + Sync,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypedSyncTool")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("input_schema", &self.input_schema)
            .finish()
    }
}

impl<T, F> TypedSyncTool<T, F>
where
    T: DeserializeOwned + Send + Sync + 'static,
    F: Fn(T, RequestHandlerExtra) -> Result<Value> + Send + Sync,
{
    /// Create a new synchronous typed tool with automatic schema generation.
    #[cfg(feature = "schema-generation")]
    pub fn new(name: impl Into<String>, handler: F) -> Self
    where
        T: JsonSchema,
    {
        let schema = generate_schema::<T>();
        Self {
            name: name.into(),
            description: None,
            input_schema: schema,
            handler,
            _phantom: PhantomData,
        }
    }

    /// Create a new synchronous typed tool with a manually provided schema.
    pub fn new_with_schema(name: impl Into<String>, schema: Value, handler: F) -> Self {
        Self {
            name: name.into(),
            description: None,
            input_schema: schema,
            handler,
            _phantom: PhantomData,
        }
    }

    /// Set the description for this tool.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[async_trait]
impl<T, F> ToolHandler for TypedSyncTool<T, F>
where
    T: DeserializeOwned + Send + Sync + 'static,
    F: Fn(T, RequestHandlerExtra) -> Result<Value> + Send + Sync,
{
    async fn handle(&self, args: Value, extra: RequestHandlerExtra) -> Result<Value> {
        // Deserialize and validate the arguments
        let typed_args: T = serde_json::from_value(args).map_err(|e| {
            crate::Error::Validation(format!("Invalid arguments for tool '{}': {}", self.name, e))
        })?;

        // Call the handler with the typed arguments
        (self.handler)(typed_args, extra)
    }

    fn metadata(&self) -> Option<ToolInfo> {
        Some(ToolInfo {
            name: self.name.clone(),
            description: self.description.clone(),
            input_schema: self.input_schema.clone(),
        })
    }
}

/// Generate a JSON schema for a type using schemars.
#[cfg(feature = "schema-generation")]
fn generate_schema<T: JsonSchema>() -> Value {
    let schema = schemars::schema_for!(T);

    // Convert the schema to JSON value
    let json_schema = serde_json::to_value(&schema).unwrap_or_else(|_| {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "additionalProperties": true
        })
    });

    // Normalize the schema by inlining $ref references
    crate::server::schema_utils::normalize_schema(json_schema)
}

/// Extension trait to add type-safe schema generation to `SimpleTool`.
pub trait SimpleToolExt {
    /// Create a `SimpleTool` with schema generated from a type.
    #[cfg(feature = "schema-generation")]
    fn with_schema_from<T: JsonSchema>(self) -> Self;
}

use super::simple_tool::SimpleTool;

impl<F> SimpleToolExt for SimpleTool<F>
where
    F: Fn(Value, RequestHandlerExtra) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>>
        + Send
        + Sync,
{
    #[cfg(feature = "schema-generation")]
    fn with_schema_from<T: JsonSchema>(self) -> Self {
        let schema = generate_schema::<T>();
        self.with_schema(schema)
    }
}

/// Extension trait to add type-safe schema generation to `SyncTool`.
pub trait SyncToolExt {
    /// Create a `SyncTool` with schema generated from a type.
    #[cfg(feature = "schema-generation")]
    fn with_schema_from<T: JsonSchema>(self) -> Self;
}

use super::simple_tool::SyncTool;

impl<F> SyncToolExt for SyncTool<F>
where
    F: Fn(Value) -> Result<Value> + Send + Sync,
{
    #[cfg(feature = "schema-generation")]
    fn with_schema_from<T: JsonSchema>(self) -> Self {
        let schema = generate_schema::<T>();
        self.with_schema(schema)
    }
}
