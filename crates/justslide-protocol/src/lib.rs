//! JustSlide protocol: the canonical command/event registry and a bounded
//! JSON-Schema subset validator. The registry JSON is the single source of
//! truth; Rust (core + MCP) embeds it and the browser chrome consumes
//! generated TypeScript (see `scripts/gen-protocol-ts.mjs`).

pub mod registry;
pub mod validate;

pub use registry::{CommandSpec, Exposure, Registry, as_json_object};
pub use validate::validate;

/// Wire protocol version spoken on the local IPC socket (`v` field).
pub const WIRE_VERSION: i64 = 1;
