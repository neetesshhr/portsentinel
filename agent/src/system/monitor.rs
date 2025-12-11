use sysinfo::{System, Disks, CpuRefreshKind, RefreshKind};
use std::{thread, time::Duration};
use port_sentinel_shared::SystemStats; // Use the shared struct!

pub fn get_system_stats() -> SystemStats {
    // 1. Configure the System Refresher
    let mut sys = System::new_with_specifics(
        RefreshKind::new().with_cpu(CpuRefreshKind::everything())
    );

    // 2. Wait a bit to collect CPU sample
    thread::sleep(Duration::from_millis(200));
    sys.refresh_cpu(); // Updated method name
    sys.refresh_memory();

    // 3. Disk Usage
    let disks = Disks::new_with_refreshed_list();
    let (disk_total, disk_used) = if let Some(disk) = disks.list().first() {
        let total = disk.total_space();
        let available = disk.available_space();
        (total, total - available)
    } else {
        (0, 0)
    };

    // 4. Return the Shared Struct
    SystemStats {
        total_memory: sys.total_memory() / 1024 / 1024,
        used_memory: sys.used_memory() / 1024 / 1024,
        total_swap: sys.total_swap() / 1024 / 1024,
        used_swap: sys.used_swap() / 1024 / 1024,
        disk_total_bytes: disk_total,
        disk_used_bytes: disk_used,
        cpu_usage: sys.global_cpu_info().cpu_usage(), // Updated method
        cpu_cores_usage: sys.cpus().iter().map(|c| c.cpu_usage()).collect(),
    }
}