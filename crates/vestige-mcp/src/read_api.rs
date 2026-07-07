//! Native loopback read-only HTTP surface (v2.2.1+)
//!
//! A small, plain-HTTP read API so a *non-MCP* consumer (e.g. a FastAPI/httpx
//! backend) can list and search Vestige memories without performing the MCP
//! streamable-HTTP `/mcp` handshake, and without coupling to Vestige's internal
//! storage layout (the back-door a consumer otherwise takes: scrolling the
//! federation Qdrant collection and filtering `source == "vestige"`).
//!
//! Design contract:
//! - **Read-only.** Only `GET` routes are mounted. No DELETE/POST/mutation, no
//!   SvelteKit SPA, no WebSocket. This is a strict subset of the full dashboard
//!   (`dashboard::mod`) and reuses its read handlers verbatim, so there is one
//!   source of truth for the response schema.
//! - **Loopback-only.** Binds `127.0.0.1` by default (override with
//!   `VESTIGE_READ_API_BIND`, mirroring `VESTIGE_HTTP_BIND`). No auth token —
//!   the loopback bind is the security boundary, consistent with the rest of the
//!   local stack (ADR-0026 singleton convention).
//! - **Opt-in.** Disabled unless `VESTIGE_READ_API_ENABLED` is `1`/`true`,
//!   matching the opt-in posture of the HTTP MCP transport and the dashboard.
//!
//! Routes (each mounted both bare and under `/api/` for consumer compatibility —
//! the legacy dashboard exposes the `/api/*` forms and an existing health probe
//! pings `http://127.0.0.1:3927/api/health`):
//!
//! | Method + path                          | Handler                     | Purpose                    |
//! |----------------------------------------|-----------------------------|----------------------------|
//! | `GET /health`, `/api/health`           | `handlers::health_check`    | readiness + memory count   |
//! | `GET /memories`, `/api/memories`       | `handlers::list_memories`   | paginated list (`?limit=&offset=&q=`) |
//! | `GET /memories/{id}`, `/api/memories/{id}` | `handlers::get_memory`  | single memory              |
//! | `GET /search`, `/api/search`           | `handlers::search_memories` | semantic/hybrid search (`?q=&limit=`) |
//! | `GET /stats`, `/api/stats`             | `handlers::get_stats`       | store statistics           |

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};

use crate::dashboard::events::VestigeEvent;
use crate::dashboard::handlers;
use crate::dashboard::state::AppState;
use vestige_core::Storage;

/// Concurrency ceiling for the read surface — mirrors the dashboard's limit.
const CONCURRENCY_LIMIT: usize = 50;

/// Default port for the read API. Reuses the well-known dashboard port (3927);
/// the two are never enabled simultaneously by the singleton, and a consumer
/// health check already targets `127.0.0.1:3927`. Override with
/// `VESTIGE_READ_API_PORT`.
pub const DEFAULT_READ_API_PORT: u16 = 3927;

/// Build the read-only axum router. `GET`-only; reuses the dashboard read
/// handlers so the response schema has a single source of truth.
pub fn build_read_router(state: AppState) -> Router {
    // Loopback bind is the security boundary; a permissive CORS layer is safe
    // here (read-only, local-only) and lets a browser-based local tool consume
    // the surface if ever needed. Only GET/OPTIONS are allowed.
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([axum::http::Method::GET, axum::http::Method::OPTIONS]);

    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/api/health", get(handlers::health_check))
        .route("/memories", get(handlers::list_memories))
        .route("/api/memories", get(handlers::list_memories))
        .route("/memories/{id}", get(handlers::get_memory))
        .route("/api/memories/{id}", get(handlers::get_memory))
        .route("/search", get(handlers::search_memories))
        .route("/api/search", get(handlers::search_memories))
        .route("/stats", get(handlers::get_stats))
        .route("/api/stats", get(handlers::get_stats))
        .layer(ServiceBuilder::new().concurrency_limit(CONCURRENCY_LIMIT).layer(cors))
        .with_state(state)
}

/// Start the read-only HTTP surface as a background task (non-blocking — use in
/// the MCP server). Cognitive engine is intentionally `None`: none of the read
/// handlers require it. `event_tx` is shared with the main bus so `/search`
/// events reach the autopilot/dashboard subscribers.
///
/// A bind failure is logged and returned (the MCP server continues without the
/// read surface), matching the dashboard's non-fatal startup posture.
pub async fn start_background(
    storage: Arc<Storage>,
    event_tx: tokio::sync::broadcast::Sender<VestigeEvent>,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState::with_event_tx(storage, None, event_tx);
    let app = build_read_router(state);

    let bind_addr: std::net::IpAddr = std::env::var("VESTIGE_READ_API_BIND")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
    let addr = SocketAddr::from((bind_addr, port));

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            warn!(
                "Read API could not bind to {}: {} (MCP server continues without read surface)",
                addr, e
            );
            return Err(Box::new(e));
        }
    };

    info!("Native read-only HTTP surface available at http://{}", addr);

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            warn!("Read API server error: {}", e);
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt; // for `oneshot`

    /// Hermetic AppState backed by a throwaway on-disk SQLite DB (an in-memory
    /// `:memory:` DB would give the reader/writer connections *separate*
    /// databases). The returned `TempDir` must be kept alive for the test's
    /// duration. Embeddings are never initialized, so no route that touches the
    /// embedding model is exercised here (the `/search` happy path is covered by
    /// the live smoke in SWAP-PROCEDURE.md against the real store).
    fn test_state() -> (AppState, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = dir.path().join("vestige-test.db");
        let storage = Arc::new(Storage::new(Some(db)).expect("temp storage"));
        (AppState::new(storage, None), dir)
    }

    async fn body_json(resp: axum::response::Response) -> serde_json::Value {
        let bytes = axum::body::to_bytes(resp.into_body(), 256 * 1024)
            .await
            .unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn health_route_returns_200_and_status_field() {
        let (state, _dir) = test_state();
        let app = build_read_router(state);
        let resp = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp).await;
        assert!(json.get("status").is_some(), "health payload has status");
        assert!(
            json.get("totalMemories").is_some(),
            "health payload has totalMemories"
        );
    }

    #[tokio::test]
    async fn api_health_alias_also_returns_200() {
        let (state, _dir) = test_state();
        let app = build_read_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn memories_route_returns_list_envelope() {
        let (state, _dir) = test_state();
        let app = build_read_router(state);
        // No `q` → `get_all_nodes` path, which does not touch embeddings.
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/memories?limit=5")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp).await;
        assert!(json.get("memories").is_some(), "list envelope has memories");
        assert!(json.get("total").is_some(), "list envelope has total");
    }

    #[tokio::test]
    async fn search_requires_q_param() {
        let (state, _dir) = test_state();
        let app = build_read_router(state);
        // Missing required `q` → axum Query rejection → 400 (extractor-level,
        // no embedding work performed).
        let resp = app
            .oneshot(Request::builder().uri("/search").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn delete_verb_is_not_routed_read_only() {
        // A DELETE to /memories/{id} must NOT be mounted on the read surface —
        // the whole point of the surface is read-only. The GET route exists, so
        // an unsupported method yields 405 (not 404), proving the path is
        // mounted GET-only.
        let (state, _dir) = test_state();
        let app = build_read_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/memories/some-id")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn post_verb_is_not_routed_read_only() {
        // No mutation verbs anywhere on the surface.
        let (state, _dir) = test_state();
        let app = build_read_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/memories")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    }
}
