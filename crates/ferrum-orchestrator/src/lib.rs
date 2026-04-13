//! # Ferrum Orchestrator
//!
//! Multi-agent orchestration layer that coordinates trading agents,
//! manages execution flow, and handles inter-agent communication.

pub mod coordinator;
pub mod message;
pub mod router;

pub use coordinator::Orchestrator;
pub use message::OrchestratorMessage;
pub use router::MessageRouter;
