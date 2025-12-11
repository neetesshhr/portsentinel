use axum::{
    extract::{Path, Query, State, Request},
    response::{IntoResponse, Redirect, Response},
    middleware::Next,
    Json, Form,
};
use axum_extra::extract::cookie::{Cookie, PrivateCookieJar};
use serde::Deserialize;
use askama::Template;
use port_sentinel_shared::{SystemStats, ProcessInfo};
use crate::state::{AppState, NodeConfig};
use std::fs;
use std::time::Duration;
use reqwest::StatusCode;

// === MIDDLEWARE ===

pub async fn auth_middleware(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    request: Request,
    next: Next,
) -> Response {
    if let Some(user_cookie) = jar.get("session_user") {
        let username = user_cookie.value();
        // We use a read lock scope here to avoid holding it during the await
        let user = {
            let users = state.auth.users.read().unwrap();
            users.iter().find(|u| u.username == username).cloned()
        };

        if let Some(user) = user {
            // Force password change logic
            if user.must_change_password {
                let path = request.uri().path();
                if path != "/change-password" && path != "/logout" {
                    return Redirect::to("/change-password").into_response();
                }
            }
            return next.run(request).await;
        }
    }
    Redirect::to("/login").into_response()
}

// === TEMPLATES ===

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate { error: Option<String> }

#[derive(Template)]
#[template(path = "change_password.html")]
struct ChangePwTemplate;

#[derive(Template)]
#[template(path = "stats.html")]
struct StatsTemplate {
    used_memory: u64, total_memory: u64, ram_pct: f64, ram_color: String,
    disk_txt: String, disk_pct: f64,
    used_swap: u64, total_swap: u64, swap_pct: f64,
    cpu_usage: f32, cpu_cores: Vec<(f64, String)>, 
    error: Option<String>,
    current_node: String,
    nodes: Vec<NodeConfig>,
    status_class: String,
    status_text: String,
}

#[derive(Template)]
#[template(path = "rows.html")]
struct RowsTemplate {
    processes: Vec<ProcessInfo>,
    trigger: String,
    current_node: String,
}

#[derive(Template)]
#[template(path = "log_modal.html")]
struct LogModalTemplate {
    pid: String, files: Vec<String>, current_node: String,
}

#[derive(Template)]
#[template(path = "log_read.html")]
struct LogReadTemplate {
    path: String, lines: Vec<String>, rate: String, current_node: String,
}

// === PAYLOAD STRUCTS ===

#[derive(Deserialize)]
pub struct AuthPayload { username: String, password: String }

#[derive(Deserialize)]
pub struct PwPayload { password: String }

#[derive(Deserialize)]
pub struct NodeParams { node: Option<String>, q: Option<String>, rate: Option<String> }

#[derive(Deserialize)]
pub struct ReadParams { path: String, rate: Option<String>, node: Option<String> }

#[derive(Deserialize)]
pub struct NodeForm { id: String, name: String, url: String, token: Option<String> }

// === HELPER FUNCTIONS (Internal) ===

fn build_client(token: Option<&String>) -> reqwest::Client {
    let mut headers = reqwest::header::HeaderMap::new();
    if let Some(t) = token {
        if let Ok(val) = reqwest::header::HeaderValue::from_str(t) {
            headers.insert("Authorization", val);
        }
    }
    reqwest::Client::builder().default_headers(headers).build().unwrap()
}

fn get_token_for_url(state: &AppState, url: &str) -> Option<String> {
    let nodes = state.nodes.read().unwrap();
    nodes.iter().find(|n| n.url == url).and_then(|n| n.token.clone())
}

async fn fetch_stats(base_url: &str, token: Option<&String>) -> Option<SystemStats> {
    let client = build_client(token);
    let url = format!("{}/api/stats", base_url);
    client.get(&url).send().await.ok()?.json::<SystemStats>().await.ok()
}

async fn fetch_processes(base_url: &str, token: Option<&String>) -> Option<Vec<ProcessInfo>> {
    let client = build_client(token);
    let url = format!("{}/api/processes", base_url);
    client.get(&url).send().await.ok()?.json::<Vec<ProcessInfo>>().await.ok()
}

async fn send_kill(base_url: &str, pid: &str, token: Option<&String>) {
    let client = build_client(token);
    let url = format!("{}/api/kill/{}", base_url, pid);
    let _ = client.post(&url).send().await;
}

async fn fetch_log_files(base_url: &str, pid: &str, token: Option<&String>) -> Vec<String> {
    let url = format!("{}/api/logs/check/{}", base_url, pid);
    let client = build_client(token);
    if let Ok(response) = client.get(&url).send().await {
        if let Ok(files) = response.json::<Vec<String>>().await { return files; }
    }
    vec![]
}

async fn fetch_log_lines(base_url: &str, path: &str, token: Option<&String>) -> Vec<String> {
    let client = build_client(token);
    let url = format!("{}/api/logs/read", base_url);
    if let Ok(response) = client.get(&url).query(&[("path", path), ("lines", "50")]).send().await {
        if let Ok(lines) = response.json::<Vec<String>>().await { return lines; }
    }
    vec!["Error reading remote log".to_string()]
}

// === HANDLERS (Public) ===

pub async fn dashboard_handler() -> impl IntoResponse {
    axum::response::Html(include_str!("../assets/index.html"))
}

pub async fn login_page() -> impl IntoResponse {
    LoginTemplate { error: None }
}

pub async fn login_submit(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Form(payload): Form<AuthPayload>
) -> impl IntoResponse {
    if state.auth.verify_user(&payload.username, &payload.password).is_some() {
        let cookie = Cookie::build(("session_user", payload.username))
            .path("/")
            .secure(false).http_only(true).build();
        let updated_jar = jar.add(cookie);
        (updated_jar, Redirect::to("/")).into_response()
    } else {
        LoginTemplate { error: Some("Invalid credentials".to_string()) }.into_response()
    }
}

pub async fn logout_handler(jar: PrivateCookieJar) -> impl IntoResponse {
    let updated_jar = jar.remove(Cookie::from("session_user"));
    (updated_jar, Redirect::to("/login"))
}

pub async fn change_password_page() -> impl IntoResponse {
    ChangePwTemplate
}

pub async fn change_password_submit(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Form(payload): Form<PwPayload>
) -> impl IntoResponse {
    if let Some(cookie) = jar.get("session_user") {
        let username = cookie.value();
        state.auth.update_password(username, &payload.password);
        return Redirect::to("/").into_response();
    }
    Redirect::to("/login").into_response()
}

pub async fn stats_handler(State(state): State<AppState>, Query(params): Query<NodeParams>) -> impl IntoResponse {
    let node_url = params.node.unwrap_or("http://127.0.0.1:3001".to_string());
    let token = get_token_for_url(&state, &node_url);
    
    // Fetch fresh list of nodes for the sidebar
    let nodes_list = state.nodes.read().unwrap().clone();
    
    match fetch_stats(&node_url, token.as_ref()).await {
        Some(stats) => {
            // === ONLINE LOGIC ===
            let ram_pct = if stats.total_memory > 0 { (stats.used_memory as f64 / stats.total_memory as f64) * 100.0 } else { 0.0 };
            let ram_color = if ram_pct > 85.0 { "bg-red-500".to_string() } else { "bg-green-500".to_string() };
            
            let disk_pct = if stats.disk_total_bytes > 0 { (stats.disk_used_bytes as f64 / stats.disk_total_bytes as f64) * 100.0 } else { 0.0 };
            fn fmt_bytes(b: u64) -> String { if b >= 1024*1024*1024 { format!("{:.1} GB", b as f64/1e9) } else { format!("{:.0} MB", b as f64/1e6) } }
            let disk_txt = format!("{} / {}", fmt_bytes(stats.disk_used_bytes), fmt_bytes(stats.disk_total_bytes));
            
            let swap_pct = if stats.total_swap > 0 { (stats.used_swap as f64 / stats.total_swap as f64) * 100.0 } else { 0.0 };
            
            let cpu_cores_data: Vec<(f64, String)> = stats.cpu_cores_usage.iter().map(|&usage| {
                let u_f64 = usage as f64;
                let color = if u_f64 > 80.0 { "bg-red-500" } else if u_f64 > 40.0 { "bg-yellow-500" } else { "bg-cyan-500" };
                (u_f64, color.to_string())
            }).collect();

            StatsTemplate {
                used_memory: stats.used_memory, total_memory: stats.total_memory, ram_pct, ram_color,
                disk_txt, disk_pct,
                used_swap: stats.used_swap, total_swap: stats.total_swap, swap_pct,
                cpu_usage: stats.cpu_usage, cpu_cores: cpu_cores_data,
                error: None, current_node: node_url, nodes: nodes_list,
                
                // Set Online Status
                status_class: "bg-green-500 animate-pulse shadow-[0_0_10px_rgba(34,197,94,0.5)]".to_string(),
                status_text: "ONLINE".to_string(),
            }
        },
        None => {
            // === OFFLINE LOGIC ===
            StatsTemplate {
                used_memory: 0, total_memory: 0, ram_pct: 0.0, ram_color: "bg-gray-500".into(),
                disk_txt: "OFFLINE".into(), disk_pct: 0.0, used_swap: 0, total_swap: 0, swap_pct: 0.0,
                cpu_usage: 0.0, cpu_cores: vec![],
                error: Some(format!("Cannot reach Agent at {}", node_url)), current_node: node_url, nodes: nodes_list,
                
                // Set Offline Status
                status_class: "bg-red-500 shadow-none".to_string(),
                status_text: "OFFLINE".to_string(),
            }
        }
    }
}

pub async fn rows_handler(State(state): State<AppState>, Query(params): Query<NodeParams>) -> impl IntoResponse {
    let node_url = params.node.unwrap_or("http://127.0.0.1:3001".to_string());
    let token = get_token_for_url(&state, &node_url);
    let processes = fetch_processes(&node_url, token.as_ref()).await.unwrap_or_default();
    let query = params.q.unwrap_or_default().to_lowercase();
    let rate_str = params.rate.unwrap_or("5".to_string());
    let filtered: Vec<ProcessInfo> = processes.into_iter().filter(|p| {
        query.is_empty() || p.name.to_lowercase().contains(&query) || p.pid.contains(&query)
    }).collect();
    let trigger = if rate_str == "0" { "refresh".to_string() } else { format!("every {}s, refresh", rate_str) };
    RowsTemplate { processes: filtered, trigger, current_node: node_url }
}

pub async fn kill_process_api(State(state): State<AppState>, Path(pid): Path<String>, Query(params): Query<NodeParams>) -> impl IntoResponse {
    let node_url = params.node.unwrap_or("http://127.0.0.1:3001".to_string());
    let token = get_token_for_url(&state, &node_url);
    send_kill(&node_url, &pid, token.as_ref()).await;
    "Signal Sent"
}

pub async fn check_logs_handler(State(state): State<AppState>, Path(pid): Path<String>, Query(params): Query<NodeParams>) -> impl IntoResponse {
    let node_url = params.node.unwrap_or("http://127.0.0.1:3001".to_string());
    let token = get_token_for_url(&state, &node_url);
    let files = fetch_log_files(&node_url, &pid, token.as_ref()).await;
    LogModalTemplate { pid, files, current_node: node_url }
}

pub async fn read_log_handler(State(state): State<AppState>, Query(params): Query<ReadParams>) -> impl IntoResponse {
    let node_url = params.node.unwrap_or("http://127.0.0.1:3001".to_string());
    let token = get_token_for_url(&state, &node_url);
    let lines = fetch_log_lines(&node_url, &params.path, token.as_ref()).await;
    let rate = params.rate.unwrap_or("2".to_string());
    LogReadTemplate { path: params.path, lines, rate, current_node: node_url }
}

pub async fn save_node_handler(State(state): State<AppState>, Json(payload): Json<NodeForm>) -> impl IntoResponse {
    let mut nodes = state.nodes.write().unwrap();
    if let Some(existing) = nodes.iter_mut().find(|n| n.id == payload.id) {
        existing.name = payload.name;
        existing.url = payload.url;
        existing.token = payload.token;
    } else {
        nodes.push(NodeConfig { id: payload.id, name: payload.name, url: payload.url, token: payload.token });
    }
    // We need to implement a public save method in main or move save_disk here
    // For simplicity, let's duplicate the small save logic here or make it public in main
    if let Ok(json) = serde_json::to_string_pretty(&*nodes) {
        let _ = fs::write("nodes.json", json);
    }
    "Saved"
}

pub async fn delete_node_handler(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let mut nodes = state.nodes.write().unwrap();
    nodes.retain(|n| n.id != id);
    if let Ok(json) = serde_json::to_string_pretty(&*nodes) {
        let _ = fs::write("nodes.json", json);
    }
    "Deleted"
}

pub async fn check_node_status(
    State(state): State<AppState>, 
    Query(params): Query<NodeParams>
) -> impl IntoResponse {
    let node_url = params.node.unwrap_or_default();
    let token = get_token_for_url(&state, &node_url);

    // 1. Setup Client with a strict timeout
    // If the agent takes >2 seconds to respond, we consider it "Laggy" or Offline
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(2000)) 
        .build()
        .unwrap();

    let url = format!("{}/api/stats", node_url);
    
    let mut request = client.get(&url);
    if let Some(t) = token {
        request = request.header("Authorization", t);
    }

    // 2. Perform the Check
    match request.send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                // === ONLINE (Green) ===
                axum::response::Html(
                    r#"<div class="w-2.5 h-2.5 rounded-full bg-green-500 animate-pulse shadow-[0_0_8px_rgba(34,197,94,0.8)]" title="Online"></div>"#
                )
            } else if resp.status() == StatusCode::UNAUTHORIZED {
                // === AUTH ERROR (Orange) ===
                // The node is UP, but our token is wrong
                axum::response::Html(
                    r#"<div class="w-2.5 h-2.5 rounded-full bg-orange-500 border border-orange-700" title="Unauthorized (Check Token)"></div>"#
                )
            } else {
                // === OTHER ERROR (Yellow) ===
                // Node is up but returning 500s or 404s
                println!("⚠️ Node {} returned status: {}", node_url, resp.status());
                axum::response::Html(
                    r#"<div class="w-2.5 h-2.5 rounded-full bg-yellow-500" title="Server Error"></div>"#
                )
            }
        },
        Err(e) => {
            // === OFFLINE (Red) ===
            // Print the specific error to the terminal so you can see WHY it failed
            println!("❌ Node Check Failed for '{}': {}", node_url, e);
            
            axum::response::Html(
                r#"<div class="w-2.5 h-2.5 rounded-full bg-red-600 border border-red-800" title="Offline"></div>"#
            )
        }
    }
}

// === SERVICE MANAGER HANDLERS ===

#[derive(Deserialize)]
pub struct ServiceParams {
    name: String,
    node: Option<String>,
}

#[derive(Template)]
#[template(path = "services.html")]
struct ServicesTemplate {
    nodes: Vec<NodeConfig>,
    current_node: String,
}

pub async fn services_page_handler(
    State(state): State<AppState>,
    Query(params): Query<NodeParams>
) -> impl IntoResponse {
    let nodes_list = state.nodes.read().unwrap().clone();
    let current_node = params.node.unwrap_or_else(|| {
        nodes_list.first().map(|n| n.url.clone()).unwrap_or("http://127.0.0.1:3001".to_string())
    });

    ServicesTemplate {
        nodes: nodes_list,
        current_node,
    }
}

// Helper to make the proxy request
async fn proxy_service_command(
    state: &AppState,
    node: Option<String>,
    path_suffix: &str,
    method: reqwest::Method,
) -> String {
    let node_url = node.unwrap_or("http://127.0.0.1:3001".to_string());
    let token = get_token_for_url(state, &node_url);
    let client = build_client(token.as_ref());
    
    let url = format!("{}/api/service/{}", node_url, path_suffix);
    
    let request = client.request(method, &url);
    
    match request.send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                resp.json::<String>().await.unwrap_or("Error parsing response".to_string())
            } else {
                format!("Error: Remote Agent returned status {}", resp.status())
            }
        },
        Err(e) => format!("Connection Error: {}", e),
    }
}

pub async fn service_status_proxy(
    State(state): State<AppState>,
    Query(params): Query<ServiceParams>
) -> impl IntoResponse {
    let suffix = format!("status/{}", params.name);
    proxy_service_command(&state, params.node, &suffix, reqwest::Method::GET).await
}

pub async fn service_start_proxy(
    State(state): State<AppState>,
    Form(params): Form<ServiceParams>
) -> impl IntoResponse {
    let suffix = format!("start/{}", params.name);
    proxy_service_command(&state, params.node, &suffix, reqwest::Method::POST).await
}

pub async fn service_stop_proxy(
    State(state): State<AppState>,
    Form(params): Form<ServiceParams>
) -> impl IntoResponse {
    let suffix = format!("stop/{}", params.name);
    proxy_service_command(&state, params.node, &suffix, reqwest::Method::POST).await
}

pub async fn service_restart_proxy(
    State(state): State<AppState>,
    Form(params): Form<ServiceParams>
) -> impl IntoResponse {
    let suffix = format!("restart/{}", params.name);
    proxy_service_command(&state, params.node, &suffix, reqwest::Method::POST).await
}