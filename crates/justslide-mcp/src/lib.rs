//! justslide-mcp: a thin local stdio MCP transport adapter for JustSlide.
//! It owns MCP session negotiation only; the running JustSlide app (Rust
//! core) owns all state, authorization and validation.
pub mod client;
pub mod endpoint;
pub mod server;
