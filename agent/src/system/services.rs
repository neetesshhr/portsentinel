use std::process::Command;
use regex::Regex;

/// Validates that the service name only contains alphanumeric characters, dots, and hyphens.
/// This prevents command injection attacks.
fn validate_service_name(name: &str) -> bool {
    // Regex: Start, alphanumeric/dot/hyphen one or more times, End.
    let re = Regex::new(r"^[a-zA-Z0-9\-\.]+$").unwrap();
    re.is_match(name)
}

pub fn get_service_status(name: &str) -> Result<String, String> {
    if !validate_service_name(name) {
        return Err("Invalid service name. Only alphanumeric, dots, and hyphens allowed.".to_string());
    }

    let output = Command::new("systemctl")
        .arg("status")
        .arg(name)
        .output()
        .map_err(|e| format!("Failed to execute systemctl: {}", e))?;

    // We return stdout if available, or stderr if it failed (systemctl returns non-zero for stopped services sometimes,
    // but we still want to see the status output)
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if stdout.is_empty() {
        Ok(stderr)
    } else {
        Ok(stdout)
    }
}

pub fn start_service(name: &str) -> Result<String, String> {
    run_service_command(name, "start")
}

pub fn stop_service(name: &str) -> Result<String, String> {
    run_service_command(name, "stop")
}

pub fn restart_service(name: &str) -> Result<String, String> {
    run_service_command(name, "restart")
}

fn run_service_command(name: &str, action: &str) -> Result<String, String> {
    if !validate_service_name(name) {
        return Err("Invalid service name. Only alphanumeric, dots, and hyphens allowed.".to_string());
    }

    let output = Command::new("systemctl")
        .arg(action)
        .arg(name)
        .output()
        .map_err(|e| format!("Failed to execute systemctl: {}", e))?;

    if output.status.success() {
        Ok(format!("Successfully execution action '{}' on service '{}'", action, name))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("Failed to {} {}: {}", action, name, stderr))
    }
}
