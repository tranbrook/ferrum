//! API server using Axum.

use crate::routes::{build_router, AppState};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

/// API server configuration
pub struct ApiServer {
    port: u16,
    state: AppState,
}

impl ApiServer {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            state: AppState { agents: vec![] },
        }
    }

    pub fn with_state(mut self, state: AppState) -> Self {
        self.state = state;
        self
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let app = build_router(self.state)
            .layer(CorsLayer::permissive())
            .layer(TraceLayer::new_for_http());

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        tracing::info!("API server listening on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}
