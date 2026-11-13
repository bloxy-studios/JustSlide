//! JustSlide core. See docs/architecture.md for the process/trust model.

pub mod core;
pub mod engine;
pub mod error;
pub mod events;
pub mod ids;
pub mod ipc;
pub mod model;
pub mod state;
pub mod store;
pub mod urls;

#[cfg(any(test, feature = "test-engine", feature = "design-fixture"))]
pub mod fixture;
#[cfg(any(test, feature = "test-engine"))]
pub mod test_engine;

pub use crate::core::{Actor, Core, PROTOCOL_VERSION, Request, Response};
pub use engine::{BrowserEngine, EngineCapabilities, EngineCommand, EngineEvent};
pub use error::{CoreError, ErrorCode};
