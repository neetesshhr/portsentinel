use std::process::Command;
use serde::{Deserialize, Serialize};
use regex::Regex;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContainerInfo {
    pub id: String,
    pub image: String,
    pub status: String,
    pub names: String,
    pub state: String, // running, exited, etc.
}

/// Validates that the container ID/Name only contains alphanumeric characters, underscores, dots, and hyphens.
fn validate_container_id(id: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z0-9\-\._]+$").unwrap();
    re.is_match(id)
}

pub fn list_containers() -> Vec<ContainerInfo> {
    // Format: {{.ID}}|{{.Image}}|{{.Status}}|{{.Names}}|{{.State}}
    let output = Command::new("docker")
        .args(["ps", "-a", "--format", "{{.ID}}|{{.Image}}|{{.Status}}|{{.Names}}|{{.State}}"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.lines().filter_map(|line| {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 5 {
                    Some(ContainerInfo {
                        id: parts[0].to_string(),
                        image: parts[1].to_string(),
                        status: parts[2].to_string(),
                        names: parts[3].to_string(),
                        state: parts[4].to_string(),
                    })
                } else {
                    None
                }
            }).collect()
        },
        _ => vec![] // Return empty if docker fails or not installed
    }
}

pub fn get_container_logs(id: &str) -> Result<Vec<String>, String> {
    if !validate_container_id(id) {
        return Err("Invalid container ID".to_string());
    }

    let output = Command::new("docker")
        .args(["logs", "--tail", "100", id]) // Fetch last 100 lines
        .output()
        .map_err(|e| e.to_string())?;

    // Docker logs often go to stderr/stdout mixed
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    
    // Combine and split lines
    let mut lines: Vec<String> = stdout.lines().map(|s| s.to_string()).collect();
    lines.extend(stderr.lines().map(|s| s.to_string()));
    
    Ok(lines)
}

pub fn control_container(id: &str, action: &str) -> Result<String, String> {
    if !validate_container_id(id) {
        return Err("Invalid container ID".to_string());
    }

    let valid_actions = ["start", "stop", "restart", "rm"];
    if !valid_actions.contains(&action) {
        return Err("Invalid action".to_string());
    }
    
    // For 'rm', usually we want -f if it's convenient, or just rm. using simple rm for now.
    let output = Command::new("docker")
        .arg(action)
        .arg(id)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(format!("Successfully executed {} on {}", action, id))
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
