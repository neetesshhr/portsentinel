use serde::{Deserialize, Serialize}; // Added Serialize
use std::fs;
use std::path::Path;

#[derive(Deserialize, Serialize, Clone, Debug)] // Added Serialize here
pub struct AgentConfig {
    pub port: u16,
    pub hostname: String,
    pub auth_token: String,
}

impl AgentConfig {
    // Now accepts a path argument
    pub fn load(path: &str) -> Self {
        // Default values
        let default_config = AgentConfig {
            port: 3001,
            hostname: "localhost".to_string(),
            auth_token: "change_me_please".to_string(),
        };

        if Path::new(path).exists() {
            match fs::read_to_string(path) {
                Ok(content) => {
                    serde_json::from_str(&content).unwrap_or_else(|_| {
                        println!("⚠️  Config file at '{}' is malformed. Using defaults.", path);
                        default_config
                    })
                }
                Err(_) => {
                    println!("⚠️  Could not read config file at '{}'. Using defaults.", path);
                    default_config
                }
            }
        } else {
            // Only generate a new file if we are using the default path "config.json"
            // If the user specified a custom path like "/etc/ps.json" and it's missing, 
            // we probably shouldn't just silently create it there without permission.
            if path == "config.json" {
                println!("ℹ️  No config found. Creating default 'config.json'.");
                let json = serde_json::to_string_pretty(&default_config).unwrap();
                let _ = fs::write(path, json);
            } else {
                println!("ℹ️  Config file '{}' not found. Using defaults in memory.", path);
            }
            default_config
        }
    }
}