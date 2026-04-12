//! Telegram bot setup and handler dispatch.

use teloxide::prelude::*;

/// Telegram bot for Ferrum
pub struct TelegramBot {
    token: String,
}

impl TelegramBot {
    pub fn new(token: String) -> Self {
        Self { token }
    }

    pub async fn run(self) {
        tracing::info!("Starting Telegram bot...");
        let bot = Bot::new(&self.token);
        
        teloxide::repl(bot, |bot: Bot, msg: Message| async move {
            let text = msg.text().unwrap_or("").to_string();
            let chat_id = msg.chat.id;
            
            let response = if text.starts_with("/help") {
                "Ferrum Trading Bot\n/help - Show help\n/list - List agents\n/start <name> - Start agent\n/stop <name> - Stop agent\n/status <name> - Agent status\n/portfolio - Portfolio\n/positions - Positions".to_string()
            } else if text.starts_with("/list") {
                "📋 Agents:\n- grid-mm (active)\n- trend-follower (paused)".to_string()
            } else if let Some(name) = text.strip_prefix("/start ") {
                format!("▶️ Starting agent: {}", name)
            } else if let Some(name) = text.strip_prefix("/stop ") {
                format!("⏹️ Stopping agent: {}", name)
            } else if let Some(name) = text.strip_prefix("/status ") {
                format!("📊 Status for {}: Active\nP&L: +5.23 USDT", name)
            } else if text.starts_with("/portfolio") {
                "💰 Portfolio:\nTotal: 1000 USDT\nP&L: +5.23 USDT\nPositions: 2".to_string()
            } else if text.starts_with("/positions") {
                "📈 Positions:\n1. BTC-USDT LONG 0.01 @ 100000\n2. ETH-USDT SHORT 0.5 @ 3500".to_string()
            } else {
                return Ok(());
            };
            
            bot.send_message(chat_id, response).await?;
            Ok(())
        }).await;
    }
}
