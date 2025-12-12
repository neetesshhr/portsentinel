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
mod db;

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
    // Create database connection pool
    // Ensure the file exists
    if !std::path::Path::new("port_sentinel.db").exists() {
        std::fs::File::create("port_sentinel.db").expect("Failed to create DB file");
    }
    
    let db_pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite:port_sentinel.db")
        .await
        .expect("Failed to connect to database");

    // Initialize Schema
    db::init_db(&db_pool).await.expect("Failed to initialize DB schema");

    // --- Data Migration (JSON -> DB) ---
    // If users table is empty, try to import from users.json
    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&db_pool)
        .await
        .unwrap_or(0);

    if user_count == 0 && std::path::Path::new("users.json").exists() {
        println!("📦 Migrating users.json to Database...");
        let users_json = std::fs::read_to_string("users.json").unwrap_or_default();
        if let Ok(users) = serde_json::from_str::<Vec<crate::auth::User>>(&users_json) {
            for user in users {
                let _ = db::create_user(&db_pool, &user).await;
            }
        }
        let _ = std::fs::rename("users.json", "users.json.bak");
    } else if user_count == 0 {
        // Create default admin user if no JSON and no DB users
        let hash = bcrypt::hash("admin", bcrypt::DEFAULT_COST).unwrap();
        let admin = crate::auth::User {
             username: "admin".to_string(), 
             password_hash: hash,
             role: "admin".to_string(),
             must_change_password: true 
        };
        let _ = db::create_user(&db_pool, &admin).await;
        println!("⚠️ No users found. Created default 'admin' user (password: admin)");
    }

    // If nodes table is empty, try to import from nodes.json
    let node_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM nodes")
        .fetch_one(&db_pool)
        .await
        .unwrap_or(0);

    if node_count == 0 && std::path::Path::new("nodes.json").exists() {
         println!("📦 Migrating nodes.json to Database...");
         let nodes_json = std::fs::read_to_string("nodes.json").unwrap_or_default();
         if let Ok(nodes) = serde_json::from_str::<Vec<crate::state::NodeConfig>>(&nodes_json) {
             for node in nodes {
                 let _ = db::upsert_node(&db_pool, &node).await;
             }
         }
         let _ = std::fs::rename("nodes.json", "nodes.json.bak");
    } else if node_count == 0 {
        println!("✨ First run detected. Auto-registering Local Agent...");
        let local_node = NodeConfig {
            id: "local".to_string(),
            name: "Local Server".to_string(),
            url: "http://127.0.0.1:3001".to_string(),
            token: None,
        };
        let _ = db::upsert_node(&db_pool, &local_node).await;
    }

    // Initialize State with DB Pool
    let key = Key::generate(); 
    let shared_state = AppState {
        db: db_pool,
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
        // === Docker Manager Routes ===
        .route("/view/containers", get(containers_page_handler))
        .route("/view/containers/list", get(containers_list_proxy))
        .route("/api/proxy/docker/logs/:id", get(docker_logs_proxy))
        .route("/api/proxy/docker/:action/:id", post(docker_control_proxy))
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