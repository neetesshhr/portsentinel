use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessInfo {
    pub pid: String,
    pub name: String,
    pub port: String,
    pub raw_line: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemStats {
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub disk_total_bytes: u64,
    pub disk_used_bytes: u64,
    pub cpu_usage: f32,
    pub cpu_cores_usage: Vec<f32>,
}