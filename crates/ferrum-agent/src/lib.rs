//! # Ferrum Agent
//!
//! OODA loop trading agent with session management and persistent learnings.

pub mod agent;
pub mod session;
pub mod learnings;
pub mod definition;

pub use agent::FerrumAgent;
pub use session::Session;
pub use learnings::LearningsStore;
pub use definition::AgentDefinitionParser;
