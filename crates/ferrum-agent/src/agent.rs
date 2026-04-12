//! Ferrum Trading Agent - OODA loop implementation.

use async_trait::async_trait;
use ferrum_core::config::{AgentConfig, AgentDefinition, RiskLimits, RiskState};
use ferrum_core::error::{FerrumError, Result};
use ferrum_core::events::FerrumEvent;
use ferrum_core::traits::*;
use ferrum_core::types::*;
use ferrum_executors::ExecutorFactory;
use ferrum_llm::{OpenAiClient, PromptBuilder};
use ferrum_risk::FerrumRiskEngine;
use ferrum_core::traits::LlmClientExt;
use crate::session::Session;
use crate::learnings::LearningsStore;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// The main Ferrum trading agent implementing the OODA loop
pub struct FerrumAgent {
    definition: AgentDefinition,
    risk_engine: Arc<FerrumRiskEngine>,
    llm: Arc<dyn LlmClient>,
    exchange: Arc<dyn ExchangeAdapter>,
    executors: HashMap<ExecutorId, Box<dyn Executor>>,
    learnings: LearningsStore,
    session: Session,
    event_tx: broadcast::Sender<FerrumEvent>,
}

impl FerrumAgent {
    pub fn new(
        definition: AgentDefinition,
        llm: Arc<dyn LlmClient>,
        exchange: Arc<dyn ExchangeAdapter>,
        event_tx: broadcast::Sender<FerrumEvent>,
    ) -> Self {
        let session = Session::new(&definition.name);
        let learnings = LearningsStore::new_in_memory();
        let risk_engine = Arc::new(FerrumRiskEngine::new());

        Self {
            definition,
            risk_engine,
            llm,
            exchange,
            executors: HashMap::new(),
            learnings,
            session,
            event_tx,
        }
    }

    pub fn with_learnings(mut self, learnings: LearningsStore) -> Self {
        self.learnings = learnings;
        self
    }

    /// Get reference to the agent definition
    pub fn definition(&self) -> &AgentDefinition { &self.definition }

    /// Get reference to the current session
    pub fn session(&self) -> &Session { &self.session }

    /// Get the risk engine
    pub fn risk_engine(&self) -> &Arc<FerrumRiskEngine> { &self.risk_engine }

    /// Get active executor count
    pub fn active_executor_count(&self) -> usize {
        self.executors.values()
            .filter(|e| matches!(e.status(), ExecutorStatus::Active | ExecutorStatus::Created))
            .count()
    }

    /// Run the main OODA loop
    pub async fn run_loop(&mut self) -> Result<()> {
        tracing::info!(
            agent = %self.definition.name,
            pair = %self.definition.config.trading_pair,
            tick_interval = self.definition.config.tick_interval_secs,
            "Starting Ferrum agent OODA loop"
        );

        loop {
            let tick_start = std::time::Instant::now();

            // Phase 0: Pre-tick risk validation (deterministic)
            let risk_state = self.risk_engine.current_state();
            if let Err(block) = self.risk_engine.validate_tick(&risk_state, &self.definition.limits) {
                tracing::warn!(reason = ?block, "Tick blocked by risk engine");
                self.session.record_tick(0, risk_state.daily_pnl,
                    format!("Tick blocked: {:?}", block));
                let _ = self.event_tx.send(FerrumEvent::AgentBlocked {
                    agent_id: self.definition.name.clone(),
                    reason: format!("{:?}", block),
                });
                tokio::time::sleep(
                    std::time::Duration::from_secs(self.definition.config.tick_interval_secs)
                ).await;
                continue;
            }

            // Phase 1: OBSERVE - Gather market data
            let observation = match self.observe().await {
                Ok(obs) => obs,
                Err(e) => {
                    tracing::error!(error = %e, "Observe phase failed");
                    tokio::time::sleep(
                        std::time::Duration::from_secs(5)
                    ).await;
                    continue;
                }
            };

            // Phase 2: ORIENT - LLM reasoning with context + learnings
            let assessment = match self.orient(&observation).await {
                Ok(a) => a,
                Err(e) => {
                    tracing::error!(error = %e, "Orient phase failed");
                    MarketAssessment {
                        regime: MarketRegime::Ranging,
                        confidence: 0.0,
                        rationale: format!("LLM error: {}", e),
                        key_factors: vec![],
                        recommended_actions: vec![],
                    }
                }
            };

            // Phase 3: DECIDE - Generate executor actions
            let actions = match self.decide(&assessment).await {
                Ok(a) => a,
                Err(e) => {
                    tracing::error!(error = %e, "Decide phase failed");
                    vec![]
                }
            };

            // Phase 4: ACT - Validate + Execute (deterministic)
            let results = self.act(actions).await.unwrap_or_default();

            // Update tick executors (process lifecycle)
            let mut events = Vec::new();
            let market = MarketData {
                connector: self.definition.config.connectors.first().cloned().unwrap_or_default(),
                pair: self.definition.config.trading_pair.clone(),
                orderbook: observation.orderbook.clone(),
                latest_candles: observation.candles.clone(),
                timestamp: chrono::Utc::now().timestamp(),
            };

            for (id, executor) in &mut self.executors {
                if let Ok(tick_events) = executor.tick(&market).await {
                    events.extend(tick_events);
                }
            }

            // Emit events
            for event in events {
                let _ = self.event_tx.send(event);
            }

            // Clean up terminated executors
            self.executors.retain(|_, e| {
                !matches!(e.status(), ExecutorStatus::Terminated(_))
            });

            // Update risk state
            let new_state = self.risk_engine.compute_state(
                risk_state.daily_pnl,
                risk_state.total_exposure,
                self.active_executor_count() as u32,
                risk_state.total_exposure, // Simplified
                risk_state.total_exposure,
                risk_state.daily_cost,
                &self.definition.limits,
            );
            self.risk_engine.update_state(new_state.clone());

            // Record tick in session
            let actions_taken = results.iter().filter(|r| matches!(r, ExecutorResult::Success { .. })).count() as u32;
            self.session.record_tick(
                actions_taken,
                new_state.daily_pnl,
                format!("Regime: {:?}, Confidence: {:.2}, Actions: {}",
                    assessment.regime, assessment.confidence, actions_taken),
            );

            let elapsed = tick_start.elapsed();
            tracing::info!(
                elapsed_ms = elapsed.as_millis(),
                regime = ?assessment.regime,
                actions = actions_taken,
                pnl = new_state.daily_pnl,
                "Tick completed"
            );

            // Sleep until next tick
            let sleep_duration = std::time::Duration::from_secs(self.definition.config.tick_interval_secs)
                .saturating_sub(elapsed);
            tokio::time::sleep(sleep_duration).await;
        }
    }

    /// Stop all executors and end session
    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!(agent = %self.definition.name, "Shutting down agent");
        for (id, executor) in &mut self.executors {
            if let Err(e) = executor.stop().await {
                tracing::error!(executor = %id.0, error = %e, "Failed to stop executor");
            }
        }
        self.session.complete();
        self.learnings.persist()?;
        Ok(())
    }
}

#[async_trait]
impl TradingAgent for FerrumAgent {
    fn agent_id(&self) -> &str { &self.definition.name }

    async fn observe(&self) -> Result<MarketObservation> {
        let pair = &self.definition.config.trading_pair;

        // Gather market data from exchange
        let orderbook = self.exchange.get_orderbook(pair).await.ok();
        let candles = self.exchange.get_candles(pair, Interval::M5, 100).await
            .unwrap_or_default();
        let positions = self.exchange.get_positions().await
            .unwrap_or_default();

        // Collect active executor metrics
        let active_executor_metrics: Vec<ExecutorMetrics> = self.executors.values()
            .map(|e| e.metrics())
            .collect();

        Ok(MarketObservation {
            orderbook,
            candles,
            positions,
            active_executor_metrics,
            risk_state: self.risk_engine.current_state(),
            sentiment_score: None, // Would be populated by RAG pipeline
            timestamp: chrono::Utc::now().timestamp(),
        })
    }

    async fn orient(&self, obs: &MarketObservation) -> Result<MarketAssessment> {
        let prompt = PromptBuilder::build_orient_prompt(
            obs,
            self.learnings.get_active(),
        );
        self.llm.structured_complete(&prompt).await
    }

    async fn decide(&self, assessment: &MarketAssessment) -> Result<Vec<ExecutorAction>> {
        let prompt = PromptBuilder::build_decide_prompt(
            assessment,
            &MarketObservation {
                orderbook: None,
                candles: vec![],
                positions: vec![],
                active_executor_metrics: vec![],
                risk_state: self.risk_engine.current_state(),
                sentiment_score: None,
                timestamp: 0,
            },
            &self.risk_engine.current_state(),
            &self.definition.limits,
        );
        self.llm.structured_complete(&prompt).await
    }

    async fn act(&self, actions: Vec<ExecutorAction>) -> Result<Vec<ExecutorResult>> {
        let mut results = Vec::new();
        let state = self.risk_engine.current_state();

        for action in &actions {
            // DETERMINISTIC validation before execution
            match self.risk_engine.validate_executor_action(action, &state, &self.definition.limits) {
                Ok(()) => {
                    // In real implementation, this would create the executor
                    // and place the order via exchange adapter
                    tracing::info!(action = ?action, "Executing action");
                    results.push(ExecutorResult::Success {
                        executor_id: ExecutorId::new(),
                    });
                }
                Err(e) => {
                    tracing::warn!(reason = ?e, "Action blocked by risk engine");
                    results.push(ExecutorResult::Blocked {
                        reason: format!("{:?}", e),
                    });
                }
            }
        }
        Ok(results)
    }
}
