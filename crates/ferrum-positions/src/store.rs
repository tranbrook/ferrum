//! Position persistence using SQLite.

use ferrum_core::error::{FerrumError, Result};
use ferrum_core::types::*;
use rusqlite::{params, Connection};
use std::path::Path;

/// SQLite-backed position store
pub struct PositionStore {
    conn: Connection,
}

impl PositionStore {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .map_err(|e| FerrumError::DatabaseError(e.to_string()))?;
        let store = Self { conn };
        store.init_tables()?;
        Ok(store)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| FerrumError::DatabaseError(e.to_string()))?;
        let store = Self { conn };
        store.init_tables()?;
        Ok(store)
    }

    fn init_tables(&self) -> Result<()> {
        self.conn.execute_batch("
            CREATE TABLE IF NOT EXISTS positions (
                id TEXT PRIMARY KEY,
                connector TEXT NOT NULL,
                base TEXT NOT NULL,
                quote TEXT NOT NULL,
                side TEXT NOT NULL,
                amount REAL NOT NULL,
                entry_price REAL NOT NULL,
                unrealized_pnl REAL DEFAULT 0,
                realized_pnl REAL DEFAULT 0,
                leverage INTEGER,
                is_lp BOOLEAN DEFAULT FALSE,
                created_at TEXT DEFAULT (datetime('now')),
                updated_at TEXT DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS trades (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                position_id TEXT REFERENCES positions(id),
                executor_id TEXT,
                side TEXT NOT NULL,
                price REAL NOT NULL,
                quantity REAL NOT NULL,
                fee REAL DEFAULT 0,
                fee_asset TEXT,
                timestamp TEXT DEFAULT (datetime('now'))
            );
        ").map_err(|e| FerrumError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub fn save_position(&self, pos: &Position) -> Result<()> {
        self.conn.execute("
            INSERT OR REPLACE INTO positions
            (id, connector, base, quote, side, amount, entry_price, unrealized_pnl, realized_pnl, leverage, is_lp)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        ", params![
            pos.id, pos.connector, pos.pair.base, pos.pair.quote,
            format!("{:?}", pos.side), pos.amount.0, pos.entry_price.0,
            pos.unrealized_pnl, pos.realized_pnl, pos.leverage, pos.is_lp
        ]).map_err(|e| FerrumError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub fn load_positions(&self) -> Result<Vec<Position>> {
        let mut stmt = self.conn.prepare("
            SELECT id, connector, base, quote, side, amount, entry_price,
                   unrealized_pnl, realized_pnl, leverage, is_lp
            FROM positions
        ").map_err(|e| FerrumError::DatabaseError(e.to_string()))?;

        let positions = stmt.query_map([], |row| {
            let side_str: String = row.get(4)?;
            let side = match side_str.as_str() {
                "Buy" => Side::Buy,
                "Sell" => Side::Sell,
                _ => Side::Range,
            };
            Ok(Position {
                id: row.get(0)?,
                connector: row.get(1)?,
                pair: TradingPair::new(
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ),
                side,
                amount: Quantity(row.get::<_, f64>(5)?),
                entry_price: Price(row.get::<_, f64>(6)?),
                unrealized_pnl: row.get(7)?,
                realized_pnl: row.get(8)?,
                leverage: row.get(9)?,
                is_lp: row.get(10)?,
            })
        }).map_err(|e| FerrumError::DatabaseError(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

        Ok(positions)
    }

    pub fn delete_position(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM positions WHERE id = ?1", params![id])
            .map_err(|e| FerrumError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub fn record_trade(&self, position_id: &str, executor_id: &str, side: &str, price: f64, quantity: f64, fee: f64) -> Result<()> {
        self.conn.execute("
            INSERT INTO trades (position_id, executor_id, side, price, quantity, fee)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        ", params![position_id, executor_id, side, price, quantity, fee])
            .map_err(|e| FerrumError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_store_crud() {
        let store = PositionStore::open_in_memory().unwrap();
        let pos = Position {
            id: "test-1".into(),
            connector: "binance".into(),
            pair: TradingPair::new("BTC", "USDT"),
            side: Side::Buy,
            amount: Quantity(0.1),
            entry_price: Price(100000.0),
            unrealized_pnl: 50.0,
            realized_pnl: 0.0,
            leverage: Some(1),
            is_lp: false,
        };
        store.save_position(&pos).unwrap();
        let loaded = store.load_positions().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "test-1");
        store.delete_position("test-1").unwrap();
        let loaded = store.load_positions().unwrap();
        assert!(loaded.is_empty());
    }
}
