//! Session management for trading agents.

use ferrum_core::error::{FerrumError, Result};
use ferrum_core::traits::{MarketAssessment, MarketObservation};
use ferrum_core::types::ExecutorResult;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;

/// A single tick record within a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickRecord {
    pub timestamp: i64,
    pub summary: String,
    pub actions_taken: u32,
    pub pnl_snapshot: f64,
}

/// A trading session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub agent_name: String,
    pub started_at: i64,
    pub ticks: Vec<TickRecord>,
    pub total_pnl: f64,
    pub total_trades: u32,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Active,
    Paused,
    Completed,
    Failed,
}

impl Session {
    pub fn new(agent_name: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            agent_name: agent_name.to_string(),
            started_at: chrono::Utc::now().timestamp(),
            ticks: Vec::new(),
            total_pnl: 0.0,
            total_trades: 0,
            status: SessionStatus::Active,
        }
    }

    pub fn record_tick(
        &mut self,
        actions_taken: u32,
        pnl_snapshot: f64,
        summary: String,
    ) {
        self.ticks.push(TickRecord {
            timestamp: chrono::Utc::now().timestamp(),
            summary,
            actions_taken,
            pnl_snapshot,
        });
        self.total_pnl = pnl_snapshot;
        self.total_trades += actions_taken;
    }

    pub fn complete(&mut self) {
        self.status = SessionStatus::Completed;
    }

    pub fn fail(&mut self, reason: &str) {
        self.status = SessionStatus::Failed;
    }

    pub fn pause(&mut self) {
        self.status = SessionStatus::Paused;
    }

    pub fn resume(&mut self) {
        self.status = SessionStatus::Active;
    }

    pub fn save_to_file(&self, dir: &Path) -> Result<()> {
        fs::create_dir_all(dir)
            .map_err(|e| FerrumError::SessionError(e.to_string()))?;
        let path = dir.join("journal.md");
        let mut content = format!(
            "# Trading Session {}\n\n## Summary\n- Agent: {}\n- Started: {}\n- Total P&L: {:.2}\n- Total Trades: {}\n- Status: {:?}\n\n## Ticks\n",
            self.id, self.agent_name, self.started_at, self.total_pnl, self.total_trades, self.status
        );
        for (i, tick) in self.ticks.iter().enumerate() {
            content.push_str(&format!(
                "### Tick {}\n- Time: {}\n- Actions: {}\n- P&L: {:.2}\n- Summary: {}\n\n",
                i + 1, tick.timestamp, tick.actions_taken, tick.pnl_snapshot, tick.summary
            ));
        }
        fs::write(path, content)
            .map_err(|e| FerrumError::SessionError(e.to_string()))
    }
}
