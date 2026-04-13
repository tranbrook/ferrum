//! Message router for inter-agent communication.

use crate::message::{AgentId, AgentRole, MessageType, OrchestratorMessage};
use std::collections::HashMap;
use tokio::sync::mpsc;

/// Routes messages between agents based on subscriptions.
pub struct MessageRouter {
    agents: HashMap<AgentId, mpsc::Sender<OrchestratorMessage>>,
    agent_roles: HashMap<AgentId, AgentRole>,
}

impl MessageRouter {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            agent_roles: HashMap::new(),
        }
    }

    /// Register an agent.
    pub fn register(&mut self, id: AgentId, role: AgentRole, sender: mpsc::Sender<OrchestratorMessage>) {
        tracing::info!("Registering agent {} ({})", id, role);
        self.agents.insert(id.clone(), sender);
        self.agent_roles.insert(id, role);
    }

    /// Unregister an agent.
    pub fn unregister(&mut self, id: &AgentId) {
        self.agents.remove(id);
        self.agent_roles.remove(id);
    }

    /// Route a message.
    pub async fn route(&self, message: OrchestratorMessage) {
        if let Some(to) = message.to.clone() {
            if let Some(sender) = self.agents.get(&to) {
                if let Err(e) = sender.send(message).await {
                    tracing::warn!("Failed to route message to {}: {}", to, e);
                }
            }
        } else {
            for (id, role) in &self.agent_roles {
                let subs = role.subscribes_to();
                if subs.contains(&message.message_type) {
                    if let Some(sender) = self.agents.get(id) {
                        if let Err(e) = sender.send(message.clone()).await {
                            tracing::warn!("Failed to broadcast to {}: {}", id, e);
                        }
                    }
                }
            }
        }
    }

    /// Get registered agent count.
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    /// List all agents.
    pub fn list_agents(&self) -> Vec<(AgentId, AgentRole)> {
        self.agent_roles.iter().map(|(id, r)| (id.clone(), *r)).collect()
    }
}

impl Default for MessageRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_router_registration_and_routing() {
        let mut router = MessageRouter::new();
        let (tx, mut rx) = mpsc::channel(16);

        router.register(
            AgentId("executor".to_string()),
            AgentRole::Executor,
            tx,
        );

        assert_eq!(router.agent_count(), 1);

        let msg = OrchestratorMessage::new(
            AgentId("analyst".to_string()),
            Some(AgentId("executor".to_string())),
            MessageType::TradeSignal,
            "Buy BTC".to_string(),
        );

        router.route(msg).await;
        let received = rx.try_recv().unwrap();
        assert_eq!(received.content, "Buy BTC");
    }
}
