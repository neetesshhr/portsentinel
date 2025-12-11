use std::process::Command;
use port_sentinel_shared::ProcessInfo; // Use shared struct

pub fn scan_ports() -> Vec<ProcessInfo> {
    let output = if cfg!(target_os = "windows") {
        Command::new("netstat")
            .args(["-ano"])
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("lsof")
            .args(["-i", "-P", "-n"])
            .output()
            .expect("failed to execute process")
    };

    let log_line = String::from_utf8(output.stdout).unwrap();
    let mut bucket = Vec::new();

    for line in log_line.lines() {
        if line.contains("COMMAND") || line.trim().is_empty() || line.contains("Active Connections") || line.contains("Proto") {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() > 1 {
            let (name, pid, port) = if cfg!(target_os = "windows") {
                ("Unknown".to_string(), parts.last().unwrap().to_string(), parts[1].to_string())
            } else {
                (parts[0].to_string(), parts[1].to_string(), parts.last().unwrap().to_string())
            };

            bucket.push(ProcessInfo {
                pid,
                name,
                port,
                raw_line: line.to_string(),
            });
        }
    }
    bucket
}

pub fn kill_process(line: &str) {
    let mut parts = line.split_whitespace();
    let pid_options = if cfg!(target_os = "windows") {
        parts.last()
    } else {
        parts.nth(1)
    };

    if let Some(pid) = pid_options {
        let mut command = if cfg!(target_os = "windows") {
            let mut cmd = Command::new("taskkill");
            cmd.args(["/F", "/PID"]);
            cmd
        } else {
            let mut cmd = Command::new("kill");
            cmd.arg("-9");
            cmd
        };
        let _ = command.arg(pid).output();
    }
}