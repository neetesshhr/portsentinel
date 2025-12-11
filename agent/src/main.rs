use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::{Path, Query, Request},
    middleware::{self, Next},
    response::Response,
    http::StatusCode,
};
use std::net::SocketAddr;
use port_sentinel_shared::{SystemStats, ProcessInfo};
use tower_http::cors::CorsLayer;
use serde::Deserialize;
use std::sync::Arc;
use clap::Parser; // Import Clap

mod system;
mod config; 

use crate::system::monitor::get_system_stats;
use crate::system::process::{scan_ports, kill_process};
use crate::system::logs::{find_process_logs, tail_log_file};
use crate::system::services::{get_service_status, start_service, stop_service, restart_service};
use crate::system::docker::{list_containers, get_container_logs, control_container, ContainerInfo};
use crate::config::AgentConfig;

// === CLI ARGUMENTS ===
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the configuration file
    #[arg(short, long, default_value = "config.json")]
    config: String,
}

// Share config across threads
struct AppState {
    config: AgentConfig,
}

#[tokio::main]
async fn main() {
    // 1. Parse Command Line Arguments
    let args = Args::parse();

    // 2. Load Configuration using the path provided (or default)
    let config = AgentConfig::load(&args.config);
    let port = config.port;
    
    println!("🔐 Agent Config Loaded from: '{}'", args.config);
    println!("   - Hostname: {}", config.hostname);
    println!("   - Port: {}", port);
    // println!("   - Auth Token: {}", config.auth_token); 

    let shared_state = Arc::new(AppState { config });

    let cors = CorsLayer::permissive();

    let app = Router::new()
        .route("/api/stats", get(stats_api))
        .route("/api/processes", get(processes_api))
        .route("/api/kill/:pid", post(kill_api))
        .route("/api/logs/check/:pid", get(logs_check_api))
        .route("/api/logs/read", get(logs_read_api))
        // === Service Control API ===
        .route("/api/service/status/:name", get(service_status_api))
        .route("/api/service/start/:name", post(service_start_api))
        .route("/api/service/stop/:name", post(service_stop_api))
        .route("/api/service/restart/:name", post(service_restart_api))
        // === Docker API ===
        .route("/api/docker/containers", get(docker_list_api))
        .route("/api/docker/logs/:id", get(docker_logs_api))
        .route("/api/docker/:action/:id", post(docker_control_api))
        .layer(cors)
        .layer(middleware::from_fn_with_state(shared_state.clone(), auth_middleware))
        .with_state(shared_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("🕵️ Agent Node Active on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// === AUTH MIDDLEWARE ===
async fn auth_middleware(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req.headers().get("Authorization");

    match auth_header {
        Some(header) if header == &state.config.auth_token => {
            Ok(next.run(req).await)
        }
        _ => {
            // println!("🚫 Unauthorized access attempt"); 
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

// === API HANDLERS ===

async fn stats_api() -> Json<SystemStats> {
    Json(get_system_stats())
}

async fn processes_api() -> Json<Vec<ProcessInfo>> {
    Json(scan_ports())
}

async fn kill_api(Path(pid): Path<String>) -> Json<String> {
    let dummy_line = format!("fake_name {}", pid);
    kill_process(&dummy_line);
    Json("Signal Sent".to_string())
}

async fn logs_check_api(Path(pid): Path<String>) -> Json<Vec<String>> {
    let files = find_process_logs(&pid);
    Json(files)
}

#[derive(Deserialize)]
struct LogReadParams { path: String, lines: Option<usize> }

async fn logs_read_api(Query(params): Query<LogReadParams>) -> Json<Vec<String>> {
    let count = params.lines.unwrap_or(50);
    let lines = tail_log_file(&params.path, count);
    Json(lines)
}

// === SERVICE HANDLERS ===

async fn service_status_api(Path(name): Path<String>) -> Result<Json<String>, StatusCode> {
    match get_service_status(&name) {
        Ok(status) => Ok(Json(status)),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

async fn service_start_api(Path(name): Path<String>) -> Result<Json<String>, StatusCode> {
    match start_service(&name) {
        Ok(msg) => Ok(Json(msg)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn service_stop_api(Path(name): Path<String>) -> Result<Json<String>, StatusCode> {
    match stop_service(&name) {
        Ok(msg) => Ok(Json(msg)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn service_restart_api(Path(name): Path<String>) -> Result<Json<String>, StatusCode> {
    match restart_service(&name) {
        Ok(msg) => Ok(Json(msg)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// === DOCKER HANDLERS ===

async fn docker_list_api() -> Json<Vec<ContainerInfo>> {
    Json(list_containers())
}

async fn docker_logs_api(Path(id): Path<String>) -> Result<Json<Vec<String>>, StatusCode> {
    match get_container_logs(&id) {
        Ok(logs) => Ok(Json(logs)),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

async fn docker_control_api(Path((action, id)): Path<(String, String)>) -> Result<Json<String>, StatusCode> {
    match control_container(&id, &action) {
        Ok(msg) => Ok(Json(msg)),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}