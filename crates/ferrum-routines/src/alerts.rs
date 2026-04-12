//! Alert management for price, volume, and funding rate events.

use ferrum_core::types::TradingPair;
use std::collections::HashMap;

/// Alert types
#[derive(Debug, Clone)]
pub enum AlertType {
    PriceAbove { pair: TradingPair, target: f64 },
    PriceBelow { pair: TradingPair, target: f64 },
    VolumeSpike { pair: TradingPair, multiplier: f64 },
    FundingRate { pair: TradingPair, threshold: f64 },
}

/// Triggered alert
#[derive(Debug, Clone)]
pub struct TriggeredAlert {
    pub alert_type: AlertType,
    pub current_value: f64,
    pub message: String,
}

/// Alert manager
pub struct AlertManager {
    alerts: HashMap<String, AlertType>,
}

impl AlertManager {
    pub fn new() -> Self { Self { alerts: HashMap::new() } }

    pub fn add(&mut self, name: String, alert: AlertType) {
        self.alerts.insert(name, alert);
    }

    pub fn remove(&mut self, name: &str) { self.alerts.remove(name); }

    pub fn check_price(&self, pair: &TradingPair, price: f64) -> Vec<TriggeredAlert> {
        self.alerts.values().filter_map(|alert| match alert {
            AlertType::PriceAbove { pair: p, target } if p == pair && price >= *target => {
                Some(TriggeredAlert {
                    alert_type: alert.clone(),
                    current_value: price,
                    message: format!("{} price {} >= target {}", pair, price, target),
                })
            }
            AlertType::PriceBelow { pair: p, target } if p == pair && price <= *target => {
                Some(TriggeredAlert {
                    alert_type: alert.clone(),
                    current_value: price,
                    message: format!("{} price {} <= target {}", pair, price, target),
                })
            }
            _ => None,
        }).collect()
    }
}

impl Default for AlertManager {
    fn default() -> Self { Self::new() }
}
