//! Multi-agent orchestrator.

use crate::message::{AgentId, AgentRole, MessageType, OrchestratorMessage};
use crate::router::MessageRouter;
use ferrum_core::error::Result;
use ferrum_core::events::FerrumEvent;
use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc};

/// Agent configuration for registration.
#[derive(Debug, Clone)]
pub struct AgentDescriptor {
    pub id: String,
    pub role: AgentRole,
    pub config: serde_json::Value,
}

/// The orchestrator manages the multi-agent trading system.
pub struct Orchestrator {
    router: MessageRouter,
    agent_senders: HashMap<AgentId, mpsc::Sender<OrchestratorMessage>>,
    event_tx: broadcast::Sender<FerrumEvent>,
    running: bool,
}

impl Orchestrator {
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(1024);
        Self {
            router: MessageRouter::new(),
            agent_senders: HashMap::new(),
            event_tx,
            running: false,
        }
    }

    /// Register a new agent with the orchestrator.
    pub fn register_agent(&mut self, descriptor: AgentDescriptor) -> mpsc::Receiver<OrchestratorMessage> {
        let id = AgentId(descriptor.id.clone());
        let role = descriptor.role;
        let (tx, rx) = mpsc::channel(256);

        self.router.register(id.clone(), role, tx.clone());
        self.agent_senders.insert(id, tx);

        tracing::info!("Registered agent: {} ({})", descriptor.id, role);
        rx
    }

    /// Remove an agent.
    pub fn unregister_agent(&mut self, id: &AgentId) {
        self.router.unregister(id);
        self.agent_senders.remove(id);
    }

    /// Send a message to a specific agent.
    pub async fn send_to(&self, to: AgentId, message: OrchestratorMessage) -> Result<()> {
        if let Some(sender) = self.agent_senders.get(&to) {
            sender.send(message).await.map_err(|e| {
                ferrum_core::error::FerrumError::AgentError(format!("Failed to send: {}", e))
            })?;
        }
        Ok(())
    }

    /// Broadcast a message to all interested agents.
    pub async fn broadcast(&self, message: OrchestratorMessage) {
        self.router.route(message).await;
    }

    /// Broadcast a FerrumEvent to all agents.
    pub fn broadcast_event(&self, event: FerrumEvent) {
        let _ = self.event_tx.send(event);
    }

    /// Subscribe to events.
    pub fn subscribe_events(&self) -> broadcast::Receiver<FerrumEvent> {
        self.event_tx.subscribe()
    }

    /// Get agent count.
    pub fn agent_count(&self) -> usize {
        self.agent_senders.len()
    }

    /// Get agents by role.
    pub fn agents_by_role(&self, role: AgentRole) -> Vec<AgentId> {
        self.router.list_agents()
            .into_iter()
            .filter(|(_, r)| *r == role)
            .map(|(id, _)| id)
            .collect()
    }

    /// Start the orchestrator.
    pub async fn start(&mut self) -> Result<()> {
        self.running = true;
        tracing::info!("Orchestrator started with {} agents", self.agent_count());
        Ok(())
    }

    /// Stop the orchestrator and all agents.
    pub async fn stop(&mut self) -> Result<()> {
        self.running = false;
        // Send stop to all agents
        for (id, sender) in &self.agent_senders {
            let stop_msg = OrchestratorMessage::new(
                AgentId("orchestrator".to_string()),
                Some(id.clone()),
                MessageType::Stop,
                "Shutdown".to_string(),
            );
            let _ = sender.send(stop_msg).await;
        }
        tracing::info!("Orchestrator stopped");
        Ok(())
    }

    /// Is the orchestrator running?
    pub fn is_running(&self) -> bool {
        self.running
    }
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}

// Static assertion: Orchestrator must be Send + Sync for use in async multi-threaded runtime
const _: () = {
    fn assert_send_sync<T: Send + Sync>() {}
    fn check() { assert_send_sync::<Orchestrator>(); }
};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_orchestrator_register_and_start() {
        let mut orch = Orchestrator::new();

        let rx = orch.register_agent(AgentDescriptor {
            id: "analyst-1".to_string(),
            role: AgentRole::Analyst,
            config: serde_json::Value::Null,
        });

        assert_eq!(orch.agent_count(), 1);
        orch.start().await.unwrap();
        assert!(orch.is_running());
    }

    #[tokio::test]
    async fn test_orchestrator_send_message() {
        let mut orch = Orchestrator::new();

        let mut rx = orch.register_agent(AgentDescriptor {
            id: "executor-1".to_string(),
            role: AgentRole::Executor,
            config: serde_json::Value::Null,
        });

        let msg = OrchestratorMessage::new(
            AgentId("analyst".to_string()),
            Some(AgentId("executor-1".to_string())),
            MessageType::TradeSignal,
            "Buy BTC".to_string(),
        );

        orch.send_to(AgentId("executor-1".to_string()), msg).await.unwrap();
        let received = rx.try_recv().unwrap();
        assert_eq!(received.content, "Buy BTC");
    }
}
