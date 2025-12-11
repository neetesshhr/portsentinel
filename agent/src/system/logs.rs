use std::process::Command;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn find_process_logs(pid: &str) -> Vec<String> {
    let output = if cfg!(target_os = "windows") {
        return vec![]; 
    } else {
        Command::new("lsof")
            .arg("-p")
            .arg(pid)
            .output()
            .expect("failed to run lsof")
    };

    let stdout = String::from_utf8(output.stdout).unwrap();
    let mut log_files = Vec::new();

    for line in stdout.lines() {
        // IMPROVED FILTER: 
        // 1. Must be a Regular File (REG)
        // 2. Must NOT be a library (.so, .dylib, .ttf)
        // 3. Must be in a log path OR end in .log/.err/.out
        if line.contains("REG") && !line.contains(".dylib") && !line.contains(".so") {
            if let Some(path) = line.split_whitespace().last() {
                
                let is_log_ext = path.ends_with(".log") || path.ends_with(".err") || path.ends_with(".out") || path.ends_with(".txt");
                let is_log_dir = path.starts_with("/var/log") || path.contains("/logs/");
                
                if is_log_ext || is_log_dir {
                    if !log_files.contains(&path.to_string()) {
                        log_files.push(path.to_string());
                    }
                }
            }
        }
    }
    log_files
}

pub fn tail_log_file(path: &str, lines: usize) -> Vec<String> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return vec![format!("Error: Cannot open file {}. (Check Permissions)", path)],
    };

    let reader = BufReader::new(file);
    // Read all lines and take the last N
    // (In production, we would use Seek to jump to end, but this is safer for now)
    let all_lines: Vec<String> = reader.lines()
        .map(|l| l.unwrap_or_else(|_| "<binary data>".to_string()))
        .collect();
    
    let start = if all_lines.len() > lines { all_lines.len() - lines } else { 0 };
    all_lines[start..].to_vec()
}