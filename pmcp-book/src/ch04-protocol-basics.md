# Chapter 4: Protocol Basics — Think Like A Website

This chapter builds intuition for MCP by mapping it to something every product team already understands: a website. Where a website is designed for humans, an MCP server is the “website for AI agents.” The same product thinking applies — navigation, forms, docs, and clear instructions — but expressed as protocol surfaces that agents can reliably use at machine speed.

The goal: after this chapter, you’ll know why every business that ships a website or app will likely ship an MCP server too, and how to design one that agents can understand and operate safely.

## A Familiar Analogy

| Website element | MCP primitive | Purpose for agents |
| --- | --- | --- |
| Home page instructions | Prompts | Set expectations, goals, and usage patterns for the agent |
| Navigation tabs | Capability discovery | Show what exists: tools, prompts, resources |
| Forms with fields | Tools (actions) | Perform operations with validated, typed arguments |
| Form labels/help text | Tool schema + descriptions | Guide correct input and communicate constraints |
| Docs/FAQ/Policies | Resources | Provide reference material the agent can read and cite |
| Notifications/toasts | Progress and events | Communicate long-running work, partial results, and completion |
| Error pages | Structured errors | Tell the agent what failed and how to fix it |

If you’ve ever improved conversions by clarifying a form label or reorganizing navigation, you already understand “XU” (the agent eXperience). Great MCP servers are great at XU.

## Visual Model

```mermaid
%%{init: { 'theme': 'neutral' }}%%
flowchart LR
  subgraph Website["Website (Human UI)"]
    home["Home page instructions"]
    nav["Navigation tabs"]
    forms["Forms + fields"]
    help["Docs/FAQ/Policies"]
    notices["Notifications"]
    errors["Error pages"]
  end

  subgraph MCP["MCP (Agent API)"]
    prompts["Prompts"]
    discovery["Discovery: tools/list, prompts/list, resources/list"]
    tools["Tools: tools/call"]
    schemas["Schemas + descriptions"]
    resources["Resources: resources/read"]
    progress["Progress + cancellation"]
    serr["Structured errors"]
  end

  home --> prompts
  nav --> discovery
  forms --> tools
  forms --> schemas
  help --> resources
  notices --> progress
  errors --> serr
```

ASCII fallback (shown if Mermaid doesn’t render):

```
[Website (Human UI)]                      [MCP (Agent API)]
  Home page instructions   --> Prompts
  Navigation tabs          --> Discovery (tools/list, prompts/list, resources/list)
  Forms + fields           --> Tools (tools/call)
  Form labels/help text    --> Schemas + descriptions
  Docs/FAQ/Policies        --> Resources (resources/read)
  Notifications/toasts     --> Progress + cancellation
  Error pages              --> Structured errors
```

## Why Build An MCP Server

- Speed: Agents operate your product without brittle scraping or slow human UI paths.
- Reliability: Strong typing and schemas reduce ambiguity and retries.
- Governance: You decide what’s allowed, audited, and measured.
- Reuse: The same server powers many agent clients, models, and runtimes.

## Core Surfaces Of The Protocol

At a high level, an MCP client connects to a server over a transport (stdio, WebSocket, or HTTP) and uses JSON-RPC methods to discover capabilities and execute actions.

- Discovery: list what exists
  - `tools/list` — enumerate available tools (your “forms”).
  - `prompts/list` and `prompts/get` — list and fetch prompt templates (your “home page instructions”).
  - `resources/list` — enumerate reference documents and data sets (your “docs”).

- Action: do work
  - `tools/call` — submit a tool with arguments (like submitting a form).

- Reading: consult context
  - `resources/read` — read a resource by id; may return text, JSON, or other content types.

- Feedback: keep the loop tight
  - Progress and cancellation — communicate long tasks and allow interruption (see Chapter 12).
  - Structured errors — explain what failed with machine-actionable details.

You’ll go deeper on each surface in Chapters 5–8. Here we focus on the mental model and design guidance that makes servers easy for agents to use.

## Resources: Docs With Relevance And Recency

Resources are your “documentation pages” for agents. Beyond the body content, include metadata that helps clients decide what to show and when to trust it.

- Priority: A number between 0.0 and 1.0 that hints at importance. Use it to surface critical references (policies, SLAs, pricing) over long-tail docs.
- Modified date: Last updated timestamp. Clients can prioritize fresher docs and display “updated on …” to guide relevance.
- Stable URIs: Treat resource URIs like permalinks; keep them stable across versions where possible.
- Small, composable docs: Prefer focused resources with clear titles over giant walls of text.

Example discovery and reading:

```json
{ "method": "resources/list", "params": {} }
```

Example response item (shape varies by client, fields shown for design intent):

```json
{
  "uri": "docs://ordering/policies/v1",
  "title": "Ordering Policies",
  "priority": 0.95,
  "modified_at": "2025-09-01T12:34:56Z",
  "mime": "text/markdown"
}
```

Then read the resource:

```json
{
  "method": "resources/read",
  "params": { "uri": "docs://ordering/policies/v1" }
}
```

Design tip: Use high priority (e.g., ≥ 0.9) for essential safety/governance docs; give experimental or niche docs a lower priority (≤ 0.3). Always set an accurate `modified_at` so clients can rank and annotate recency.

## Prompts: User‑Controlled Workflows

Prompts are structured instructions exposed by the server and discovered by clients. They act like “guided workflows” that users can explicitly select in the UI.

- User controlled: Prompts are intended for user initiation from the client UI, not silent auto‑execution.
- Discoverable: Clients call `prompts/list` to surface available workflows; each prompt advertises arguments and a description.
- Templated: Clients call `prompts/get` with arguments to expand into concrete model messages.
- Workflow fit: Design prompts to match common user journeys (e.g., “Refund Order”, “Create Support Ticket”, “Compose Quote”).

Discover prompts:

```json
{ "method": "prompts/list", "params": {} }
```

Example prompt metadata (shape for illustration):

```json
{
  "name": "refund_order",
  "description": "Guide the agent to safely process a refund.",
  "arguments": [
    { "name": "order_id", "type": "string", "required": true },
    { "name": "reason", "type": "string", "required": false }
  ]
}
```

Get a concrete prompt with arguments:

```json
{
  "method": "prompts/get",
  "params": {
    "name": "refund_order",
    "arguments": { "order_id": "ord_123", "reason": "damaged" }
  }
}
```

XU tip: Treat prompts like your website’s primary CTAs — few, clear, and high‑signal. Link prompts to relevant resources (policies, SLAs) by stable URI so the agent can cite and comply.

## Designing For XU (Agent eXperience)

Treat agents like highly efficient, literal users. Design with the same rigor you would for a public-facing product.

- Clear names: Prefer verbs and domain terms (`create_order`, `refund_payment`).
- Tight schemas: Mark required vs optional, use enums, bounds, patterns, and example values.
- Helpful descriptions: Document constraints (currency, time zone, rate limits) where the agent needs them.
- Idempotency: Make retries safe; include idempotency keys where appropriate.
- Determinism first: Avoid side effects that depend on hidden state whenever possible.
- Predictable errors: Use specific error codes/messages and suggest next actions.
- Resource-first docs: Publish policies, SLAs, product catalogs, and changelogs as `resources/*`.
- Versioning: Introduce new tools instead of silently changing semantics; deprecate old ones gently.

## End-to-End Flow (Website Lens)

1) Land on “home” → The agent discovers `prompts` and reads guidance on how to use the server.

2) Browse navigation → The agent calls `tools/list`, `resources/list`, and optionally `prompts/list` to map the surface area.

3) Open a form → The agent selects a tool, reads its schema and descriptions, and prepares arguments.

4) Submit the form → The agent calls `tools/call` with structured arguments.

5) Watch progress → The server emits progress updates and finally returns a result or error.

6) Read the docs → The agent calls `resources/read` to consult policies, FAQs, or domain data.

7) Recover gracefully → On error, the agent adjusts inputs based on structured feedback and retries.

### Sequence Walkthrough

```mermaid
%%{init: { 'theme': 'neutral' }}%%
sequenceDiagram
  autonumber
  participant A as Agent Client
  participant S as MCP Server

  A->>S: prompts/list
  S-->>A: available prompts
  A->>S: tools/list, resources/list
  S-->>A: tool schemas, resource URIs

  A->>S: tools/call create_order(args)
  rect rgb(245,245,245)
    S-->>A: progress events (optional)
  end
  S-->>A: result or structured error

  A->>S: resources/read docs://ordering/policies/v1
  S-->>A: policy text/JSON
```

ASCII fallback:

```
Agent Client -> MCP Server: prompts/list
MCP Server  -> Agent Client: available prompts
Agent Client -> MCP Server: tools/list, resources/list
MCP Server  -> Agent Client: tool schemas, resource URIs

Agent Client -> MCP Server: tools/call create_order(args)
MCP Server  -> Agent Client: progress: "validating", 20%
MCP Server  -> Agent Client: progress: "placing order", 80%
MCP Server  -> Agent Client: result or structured error

Agent Client -> MCP Server: resources/read docs://ordering/policies/v1
MCP Server  -> Agent Client: policy text/JSON
```

## Minimal JSON Examples

List tools (like “navigation tabs”):

```json
{ "method": "tools/list", "params": {} }
```

Call a tool (like submitting a form):

```json
{
  "method": "tools/call",
  "params": {
    "name": "create_order",
    "arguments": {
      "customer_id": "cus_123",
      "items": [
        { "sku": "SKU-42", "qty": 2 },
        { "sku": "SKU-99", "qty": 1 }
      ],
      "currency": "USD"
    }
  }
}
```

Read a resource (like opening docs):

```json
{
  "method": "resources/read",
  "params": { "uri": "docs://ordering/policies/v1" }
}
```

## Bootstrap With PMCP (Rust)

Here’s a sketch of a website-like MCP server using PMCP. We’ll add full coverage of tools, resources, and prompts in later chapters; this snippet focuses on the “forms” (tools) first.

```rust
use pmcp::{Server, ToolHandler, RequestHandlerExtra, Result};
use serde_json::{json, Value};
use async_trait::async_trait;

// Form 1: Search products
struct SearchProducts;

#[async_trait]
impl ToolHandler for SearchProducts {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10);

        // In a real server, query your catalog here.
        Ok(json!({
            "content": [{ "type": "text", "text": format!(
                "Found results for '{}' (limit {})", query, limit
            )}],
            "isError": false
        }))
    }
}

// Form 2: Create order
struct CreateOrder;

#[async_trait]
impl ToolHandler for CreateOrder {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        // Validate required fields (ids, items, currency, etc.)
        if args.get("customer_id").and_then(|v| v.as_str()).is_none() {
            return Err(pmcp::Error::validation("Missing 'customer_id'"));
        }
        // Normally you would validate items/currency and write to storage.

        Ok(json!({
            "content": [{ "type": "text", "text": "Order created (demo)" }],
            "isError": false
        }))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let server = Server::builder()
        .name("website-like-server")
        .version("0.1.0")
        .tool("search_products", SearchProducts)
        .tool("create_order", CreateOrder)
        .build()?;

    server.run_stdio().await
}
```

As you evolve this server, bake in XU:

- Add precise schemas and descriptions to each tool (Chapter 5).
- Publish a small set of reference docs as `resources/*` with stable URIs (Chapter 6).
- Provide starter prompts that teach the agent ideal workflows (Chapter 7).
- Return actionable errors and support progress/cancellation for long tasks (Chapters 8 and 12).

## Checklist: From Website To MCP

- Map key user journeys → tools with clear names and schemas.
- Extract onboarding docs/FAQs → resources with stable IDs.
- Translate “how to use our product” → prompts that set expectations and rules.
- Define safe defaults, rate limits, and idempotency.
- Log and measure usage to improve XU.

## Where To Go Next

- Tools & Tool Handlers (Chapter 5)
- Resources & Resource Management (Chapter 6)
- Prompts & Templates (Chapter 7)
- Error Handling & Recovery (Chapter 8)
- Progress Tracking & Cancellation (Chapter 12)
