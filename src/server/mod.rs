//! Web server for HTTP API and UI

use crate::clients::OllamaClient;
use crate::config::Config;
use crate::domain::SearchResult;
use crate::error::Result;
use crate::repositories::VectorStore;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub ollama: Arc<OllamaClient>,
}

impl AppState {
    pub fn new(config: Config, ollama: OllamaClient) -> Self {
        Self {
            config,
            ollama: Arc::new(ollama),
        }
    }
}

/// Start the web server
pub async fn serve(host: String, port: u16, config: Config) -> Result<()> {
    info!("Starting web server on {}:{}", host, port);

    // Initialize Ollama client
    let ollama = OllamaClient::new(
        config.ollama.base_url.clone(),
        config.ollama.timeout_seconds,
    )?;

    let state = AppState::new(config, ollama);

    // Build routes
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/build-info.js", get(build_info_handler))
        .route("/favicon.ico", get(favicon_handler))
        .route("/api/health", get(health_handler))
        .route("/api/stats", get(stats_handler))
        .route("/api/search", get(search_handler))
        .route("/api/models", get(models_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Bind and serve
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("Server listening on http://{}", addr);
    info!("API documentation available at http://{}/api/health", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

// ============================================================================
// Handlers
// ============================================================================

/// Root handler - returns simple HTML UI
async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../../static/index.html"))
}

/// Build info handler - returns JavaScript with build information
async fn build_info_handler() -> Response {
    // Try to read the build-info.js file, fallback to default if not found
    let build_info = std::fs::read_to_string("static/build-info.js")
        .unwrap_or_else(|_| {
            "window.BUILD_INFO = { host: 'unknown', commit: 'unknown', timestamp: 'unknown' };".to_string()
        });

    (
        StatusCode::OK,
        [("Content-Type", "application/javascript")],
        build_info,
    )
        .into_response()
}

/// Favicon handler - serves favicon.ico from static directory
async fn favicon_handler() -> Response {
    match std::fs::read("static/favicon.ico") {
        Ok(data) => (
            StatusCode::OK,
            [("Content-Type", "image/x-icon")],
            data,
        )
            .into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Health check endpoint
async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    let ollama_healthy = state.ollama.health_check().await.unwrap_or(false);

    let health = HealthResponse {
        status: "ok".to_string(),
        ollama_available: ollama_healthy,
    };

    Json(health)
}

/// Statistics endpoint
async fn stats_handler(State(state): State<AppState>) -> Response {
    // Create a new connection for this request
    let store = match VectorStore::new(&state.config.database.path) {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to open database: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    match store.get_stats() {
        Ok(stats) => {
            let response = StatsResponse {
                document_count: stats.document_count,
                chunk_count: stats.chunk_count,
                embedding_count: stats.embedding_count,
                db_size_bytes: stats.db_size_bytes,
            };
            Json(response).into_response()
        }
        Err(e) => {
            warn!("Failed to get stats: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Search endpoint
async fn search_handler(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Response {
    if params.query.is_empty() {
        return (StatusCode::BAD_REQUEST, "Query parameter is required").into_response();
    }

    // Generate the query embedding first (this is the async part)
    let model = state.config.ollama.default_model.clone();
    let query_embedding = match state.ollama.embed(&model, &params.query).await {
        Ok(emb) => emb,
        Err(e) => {
            warn!("Failed to generate embedding: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    // Now do the database search (synchronous, doesn't cross await)
    let store = match VectorStore::new(&state.config.database.path) {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to open database: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    let mut results = match store.search_similar(&query_embedding, &model, params.top_k) {
        Ok(r) => r,
        Err(e) => {
            warn!("Search failed: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    // Filter by threshold
    if params.threshold > 0.0 {
        results.retain(|r| r.similarity >= params.threshold);
    }

    let response: Vec<SearchResultResponse> =
        results.iter().map(SearchResultResponse::from).collect();
    Json(response).into_response()
}

/// Models endpoint
async fn models_handler(State(state): State<AppState>) -> Response {
    match state.ollama.list_models().await {
        Ok(models) => {
            let response: Vec<ModelResponse> = models
                .iter()
                .map(|m| ModelResponse {
                    name: m.name.clone(),
                    size: m.size,
                    modified_at: m.modified_at.clone(),
                })
                .collect();
            Json(response).into_response()
        }
        Err(e) => {
            warn!("Failed to list models: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
struct SearchQuery {
    query: String,
    #[serde(default = "default_top_k")]
    top_k: usize,
    #[serde(default)]
    threshold: f32,
}

fn default_top_k() -> usize {
    10
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    ollama_available: bool,
}

#[derive(Debug, Serialize)]
struct StatsResponse {
    document_count: i64,
    chunk_count: i64,
    embedding_count: i64,
    db_size_bytes: i64,
}

#[derive(Debug, Serialize)]
struct SearchResultResponse {
    source: String,
    chunk_index: usize,
    content: String,
    similarity: f32,
}

impl From<&SearchResult> for SearchResultResponse {
    fn from(result: &SearchResult) -> Self {
        Self {
            source: result.document.source.clone(),
            chunk_index: result.chunk.chunk_index,
            content: result.chunk.content.clone(),
            similarity: result.similarity,
        }
    }
}

#[derive(Debug, Serialize)]
struct ModelResponse {
    name: String,
    size: u64,
    modified_at: String,
}
