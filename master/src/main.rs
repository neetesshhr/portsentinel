use axum::{
    routing::{get, post},
    Router,
    middleware,
};
use clap::Parser;
use tower_http::services::{ServeDir, ServeFile};
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::fs;
use axum_extra::extract::cookie::Key;

// Modules
mod auth;
mod state;
mod handlers;

use crate::auth::AuthState;
use crate::state::{AppState, NodeConfig};
use crate::handlers::*; 



const NODES_FILE: &str = "nodes.json";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to run the dashboard on
    #[arg(short, long, default_value_t = 7878)]
    port: u16,
}

fn load_nodes_from_disk() -> Vec<NodeConfig> {
    if let Ok(content) = fs::read_to_string(NODES_FILE) {
        serde_json::from_str(&content).unwrap_or_else(|_| vec![default_node()])
    } else {
        vec![default_node()]
    }
}

fn default_node() -> NodeConfig {
    NodeConfig { 
        id: "local".to_string(), 
        name: "Local Agent".to_string(), 
        url: "http://127.0.0.1:3001".to_string(),
        token: Some("change_me_please".to_string()),
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    // Determine the assets directory path
    let assets_path = if std::path::Path::new("master/assets").exists() {
        "master/assets"
    } else {
        "assets"
    };
    let serve_dir = ServeDir::new(assets_path);
    println!("📂 Serving assets from: {}", assets_path);

    // --- Initialization ---
    let auth_state = AuthState::load();
    let initial_nodes = load_nodes_from_disk();
    let key = Key::generate(); 

    let shared_state = AppState {
        nodes: Arc::new(RwLock::new(initial_nodes)),
        auth: auth_state,
        key,
    };

    // --- 1. Protected Router ---
    // These routes REQUIRE the auth_middleware
    let protected_routes = Router::new()
        .route("/", get(dashboard_handler))
        .route("/view/rows", get(rows_handler))
        .route("/view/stats", get(stats_handler))
        .route("/kill/:pid", post(kill_process_api))
        .route("/logs/check/:pid", get(check_logs_handler))
        .route("/logs/read", get(read_log_handler))
        .route("/api/nodes/save", post(save_node_handler))
        .route("/api/nodes/delete/:id", post(delete_node_handler))
        .route("/api/check-status", get(check_node_status))
        // === Service Manager Routes ===
        .route("/view/services", get(services_page_handler))
        .route("/api/proxy/service/status", get(service_status_proxy))
        .route("/api/proxy/service/start", post(service_start_proxy))
        .route("/api/proxy/service/stop", post(service_stop_proxy))
        .route("/api/proxy/service/restart", post(service_restart_proxy))
        // We apply the layer ONLY to this router block
        // We use from_fn_with_state to inject the state into the middleware
        .layer(middleware::from_fn_with_state(shared_state.clone(), auth_middleware));

    // --- 2. Public Router ---
    // These routes do NOT pass through auth_middleware
    let public_routes = Router::new()
        .route("/login", get(login_page).post(login_submit))
        .route("/logout", get(logout_handler))
        .route("/change-password", get(change_password_page).post(change_password_submit))
        .nest_service("/assets", serve_dir);

    // --- 3. Merge & Launch ---
    let app = Router::new()
        .merge(protected_routes)
        .merge(public_routes)
        .with_state(shared_state);

        let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
        println!("🚀 Master Dashboard available at http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}