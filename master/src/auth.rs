use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::{Arc, RwLock};
use bcrypt::{hash, verify, DEFAULT_COST};

const USERS_FILE: &str = "users.json";

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct User {
    pub username: String,
    pub password_hash: String,
    pub must_change_password: bool,
}

#[derive(Clone)]
pub struct AuthState {
    pub users: Arc<RwLock<Vec<User>>>,
}

impl AuthState {
    // Load users from disk or create default admin/admin
    pub fn load() -> Self {
        let users = if let Ok(content) = fs::read_to_string(USERS_FILE) {
            serde_json::from_str(&content).unwrap_or_else(|_| vec![Self::default_user()])
        } else {
            let default = Self::default_user();
            Self::save_to_disk(&vec![default.clone()]);
            vec![default]
        };
        AuthState { users: Arc::new(RwLock::new(users)) }
    }

    fn default_user() -> User {
        // Default password is "admin"
        let hash = hash("admin", DEFAULT_COST).expect("Failed to hash default password");
        User {
            username: "admin".to_string(),
            password_hash: hash,
            must_change_password: true, // Forces change on first login
        }
    }

    pub fn save(&self) {
        let users = self.users.read().unwrap();
        Self::save_to_disk(&users);
    }

    fn save_to_disk(users: &[User]) {
        if let Ok(json) = serde_json::to_string_pretty(users) {
            let _ = fs::write(USERS_FILE, json);
        }
    }

    pub fn verify_user(&self, username: &str, password: &str) -> Option<User> {
        let users = self.users.read().unwrap();
        if let Some(user) = users.iter().find(|u| u.username == username) {
            if verify(password, &user.password_hash).unwrap_or(false) {
                return Some(user.clone());
            }
        }
        None
    }

    pub fn update_password(&self, username: &str, new_password: &str) {
        let mut users = self.users.write().unwrap();
        if let Some(user) = users.iter_mut().find(|u| u.username == username) {
            user.password_hash = hash(new_password, DEFAULT_COST).unwrap();
            user.must_change_password = false; // Flag cleared!
        }
        // Save immediately
        Self::save_to_disk(&users);
    }
}