# TypeScript MCP SDK JSON-RPC Message Handling Investigation

## Overview

This investigation analyzes how the TypeScript MCP SDK handles JSON-RPC messages, particularly focusing on the `tools/list` method routing, request/response flow, and error handling mechanisms.

## Key Findings

### 1. JSON-RPC Message Structure

The TypeScript SDK follows JSON-RPC 2.0 specification with the following message types:

#### Request Message Format
```typescript
{
  "jsonrpc": "2.0",
  "id": number | string,  // Unique request ID
  "method": string,       // Method name like "tools/list"
  "params": {             // Optional parameters
    "_meta": {            // Optional metadata
      "progressToken": number | string
    },
    // ... method-specific parameters
  }
}
```

#### Response Message Format
```typescript
// Success response
{
  "jsonrpc": "2.0",
  "id": number | string,  // Matching request ID
  "result": {             // Method-specific result
    // ... response data
  }
}

// Error response
{
  "jsonrpc": "2.0",
  "id": number | string,  // Matching request ID
  "error": {
    "code": number,       // Error code (e.g., -32601 for Method Not Found)
    "message": string,    // Error description
    "data": any           // Optional additional error data
  }
}
```

### 2. Message Routing Architecture

The message routing follows this flow:

```
Transport → Protocol._onmessage → Protocol._onrequest → Handler
```

#### Core Routing Components

1. **Protocol Class** (`src/shared/protocol.ts`):
   - Base class that handles JSON-RPC message routing
   - Maintains `_requestHandlers: Map<string, handler>` for method routing
   - Message type detection using type guards:
     - `isJSONRPCRequest()`
     - `isJSONRPCResponse()`
     - `isJSONRPCNotification()`

2. **Server Class** (`src/server/index.ts`):
   - Extends Protocol class
   - Handles MCP-specific capability validation
   - Manages server lifecycle and initialization

3. **McpServer Class** (`src/server/mcp.ts`):
   - High-level abstraction over Server class
   - Auto-registers standard MCP handlers including `tools/list`

#### Request Handler Registration

```typescript
// Method handlers are registered using schemas
this.server.setRequestHandler(
  ListToolsRequestSchema,  // Schema defines method: "tools/list"
  (request): ListToolsResult => {
    // Handler implementation
    return { tools: [...] };
  }
);
```

### 3. Tools/List Method Implementation

#### Schema Definition
```typescript
// In types.ts
export const ListToolsRequestSchema = PaginatedRequestSchema.extend({
  method: z.literal("tools/list"),
});

export const ListToolsResultSchema = PaginatedResultSchema.extend({
  tools: z.array(ToolSchema),
});
```

#### Handler Implementation (from `src/server/mcp.ts`)
```typescript
this.server.setRequestHandler(
  ListToolsRequestSchema,
  (): ListToolsResult => ({
    tools: Object.entries(this._registeredTools)
      .filter(([, tool]) => tool.enabled)
      .map(([name, tool]): Tool => ({
        name,
        title: tool.title,
        description: tool.description,
        inputSchema: tool.inputSchema 
          ? zodToJsonSchema(tool.inputSchema) 
          : EMPTY_OBJECT_JSON_SCHEMA,
        annotations: tool.annotations,
        _meta: tool._meta,
      })),
  })
);
```

#### Tool Schema Structure
```typescript
export const ToolSchema = BaseMetadataSchema.extend({
  description: z.optional(z.string()),
  inputSchema: z.object({
    type: z.literal("object"),
    properties: z.optional(z.object({}).passthrough()),
    required: z.optional(z.array(z.string())),
  }).passthrough(),
  outputSchema: z.optional(z.object({}).passthrough()),
});

// BaseMetadataSchema provides:
export const BaseMetadataSchema = z.object({
  name: z.string(),          // Tool identifier
  title: z.optional(z.string()), // Display name
}).passthrough();
```

### 4. Error Handling for Unknown Methods

When a method is not found, the Protocol class handles it in `_onrequest()`:

```typescript
private _onrequest(request: JSONRPCRequest, extra?: MessageExtraInfo): void {
  const handler = this._requestHandlers.get(request.method) ?? this.fallbackRequestHandler;

  if (handler === undefined) {
    capturedTransport?.send({
      jsonrpc: "2.0",
      id: request.id,
      error: {
        code: ErrorCode.MethodNotFound,  // -32601
        message: "Method not found",
      },
    });
    return;
  }
  // ... continue with handler execution
}
```

#### Error Codes
```typescript
export enum ErrorCode {
  // SDK error codes
  ConnectionClosed = -32000,
  RequestTimeout = -32001,
  
  // Standard JSON-RPC error codes
  ParseError = -32700,
  InvalidRequest = -32600,
  MethodNotFound = -32601,    // Used for unknown methods
  InvalidParams = -32602,
  InternalError = -32603,
}
```

### 5. Message Flow Example

For a `tools/list` request:

1. **Client sends request**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/list",
  "params": {}
}
```

2. **Server processes**:
   - Transport receives raw message
   - Protocol._onmessage identifies as JSONRPCRequest
   - Protocol._onrequest looks up "tools/list" handler
   - Handler executes and returns ListToolsResult
   - Protocol sends response

3. **Server responds**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "tools": [
      {
        "name": "summarize",
        "description": "Summarize any text using an LLM",
        "inputSchema": {
          "type": "object",
          "properties": {
            "text": {
              "type": "string",
              "description": "Text to summarize"
            }
          },
          "required": ["text"]
        }
      }
    ]
  }
}
```

4. **Unknown method error example**:
```json
// Request
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "unknown/method"
}

// Response
{
  "jsonrpc": "2.0",
  "id": 2,
  "error": {
    "code": -32601,
    "message": "Method not found"
  }
}
```

### 6. Key Implementation Details

#### Handler Registration Process
- Handlers are registered using Zod schemas that define both method name and parameter structure
- The `setRequestHandler()` method extracts the method name from the schema
- Capability validation ensures the server declares support for the method
- Input validation occurs automatically using the schema

#### Request Processing Pipeline
1. **Message Type Detection**: Uses type guards to identify request vs response vs notification
2. **Method Routing**: Maps method string to registered handler function
3. **Parameter Validation**: Zod schema validates request parameters
4. **Handler Execution**: Calls registered handler with validated parameters
5. **Response Formatting**: Wraps result in JSON-RPC response structure
6. **Error Handling**: Catches exceptions and formats as JSON-RPC errors

#### Capability Management
- Server declares capabilities during initialization
- Client capabilities are stored after handshake
- Methods validate required capabilities before execution
- Tools capability enables both `tools/list` and `tools/call` methods

## Comparison Points for Rust Implementation

1. **Method Registration**: TypeScript uses Zod schemas for both validation and method identification
2. **Error Handling**: Standardized JSON-RPC error codes with structured error responses  
3. **Handler Signature**: Handlers receive parsed/validated parameters and execution context
4. **Capability Validation**: Both client and server capabilities are checked before method execution
5. **Response Structure**: Strict adherence to JSON-RPC 2.0 specification
6. **Type Safety**: Zod provides runtime type validation matching TypeScript types

## Files Analyzed

- `/Users/guy/Development/mcp/sdk/typescript-sdk/src/shared/protocol.ts` - Core JSON-RPC protocol implementation
- `/Users/guy/Development/mcp/sdk/typescript-sdk/src/server/index.ts` - MCP Server base class
- `/Users/guy/Development/mcp/sdk/typescript-sdk/src/server/mcp.ts` - High-level MCP server with auto-registration
- `/Users/guy/Development/mcp/sdk/typescript-sdk/src/types.ts` - Message schemas and type definitions
- `/Users/guy/Development/mcp/sdk/typescript-sdk/src/examples/server/toolWithSampleServer.ts` - Example server implementation