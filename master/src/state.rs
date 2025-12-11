use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};
use axum_extra::extract::cookie::Key;
use axum::extract::FromRef;
use crate::auth::AuthState;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct NodeConfig {
    pub id: String,
    pub name: String,
    pub url: String,
    pub token: Option<String>, 
}

#[derive(Clone)]
pub struct AppState {
    pub nodes: Arc<RwLock<Vec<NodeConfig>>>,
    pub auth: AuthState,
    pub key: Key,
}

// This allows the PrivateCookieJar to extract the Key from AppState
impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}