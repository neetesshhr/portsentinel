<div align="center">

# 🛡️ PortSentinel
### The Lightweight, Self-Hosted Infrastructure Monitor

![Version](https://img.shields.io/badge/version-1.0.0-blue.svg?style=flat-square)
![License](https://img.shields.io/badge/license-GPL-green.svg?style=flat-square)
![Rust](https://img.shields.io/badge/built_with-Rust-orange.svg?style=flat-square)
![Docker](https://img.shields.io/badge/docker-ready-blue.svg?style=flat-square)

</div>

---

**PortSentinel** is a modern, high-performance monitoring solution designed for sysadmins who want speed, simplicity, and complete data ownership. Unlike heavy enterprise agents, PortSentinel runs on a fraction of the resources while providing a beautiful, real-time dashboard.

## ✨ Why PortSentinel?

*   🚀 **Blazing Fast**: Written in **Rust**, the agent consumes negligible CPU and <20MB RAM.
*   📦 **Zero Dependencies**: Static binaries. No Python, No Node.js, No JVM required on your servers.
*   🎨 **Modern UI**: A dark-mode, responsive dashboard built with HTMX for real-time updates.
*   🔐 **Self-Hosted**: Your data never leaves your network. Now with SQLite storage.
*   🐋 **Docker First**: First-class support for monitoring and controlling Docker containers.

---

## 📸 Dashboard Preview

| **Real-Time Dashboard** | **Service Manager** |
|:---:|:---:|
| *(Place screenshot of Dashboard here)* | *(Place screenshot of Service Manager here)* |

---

## 📥 Installation

### Option 1: The All-in-One Installer (Recommended)
We provide a universal installer that can set up the Master Dashboard or just the Agent.

1.  **Download** the latest release bundle (`port_sentinel_bundle_x86_64.tar.gz`) to your server.
2.  **Extract and Run**:
    ```bash
    tar -xzf port_sentinel_bundle_x86_64.tar.gz
    cd dist
    
    # To install Master Dashboard + Agent (Main Server):
    sudo ./install.sh --master
    
    # To install Agent Only (Remote Nodes):
    sudo ./install.sh --agent
    ```
3.  **Access**: Open `http://<YOUR_IP>:7878`. The local agent is automatically registered!

### Option 2: Docker
Prefer containers? We've got you covered.

**Run Everything (Master + Agent):**
```bash
docker-compose up -d
```
*Note: To monitor the host system (processes/services) from Docker, the container runs in privileged mode.*

---

## 💻 System Requirements (Master Node)

PortSentinel is extremely resource-efficient.

| Resource | Minimum | Recommended (50+ Nodes) |
| :--- | :--- | :--- |
| **CPU** | 1 vCPU | 2 vCPU |
| **RAM** | 512 MB | 2 GB |
| **Disk** | 100 MB | 10 GB SSD (for Logs) |
| **OS** | Any Linux (x86_64) | Ubuntu / Debian / Alpine |

---

## 🔋 Features

### 🖥️ System Monitoring
*   Real-time CPU, Memory, Swap, and Disk usage.
*   Historical tracking (Coming in Enterprise Edition).

### 🛠️ Service Manager
*   Control `systemd` services remotely.
*   **Actions**: Status, Start, Stop, Restart.
*   *Security*: Strict input validation ensures only safe service names are processed.

### 🐳 Docker Manager
*   List all containers on connected nodes.
*   View container logs in real-time.
*   Restart, Stop, or Start containers from the dashboard.

### 🕵️ Process Inspector
*   View top consumers (CPU/RAM).
*   Kill runaway processes.
*   Inspect open files and network connections (`lsof`).

---

## ⚙️ Configuration

The application uses `config.json` (if present) or Environment Variables.

| Variable | Description | Default |
| :--- | :--- | :--- |
| `PORT` | Web Server Port | `7878` (Master), `3001` (Agent) |
| `DATABASE_URL` | SQLite Connection String | `sqlite:port_sentinel.db` |
| `AUTH_TOKEN` | Agent Shared Secret | `None` (Open) |

---

## 🤝 Contributing

We welcome contributions! Please see `CONTRIBUTING.md` for details on how to set up the development environment.

## 📄 License

GPL v3 © 2024 PortSentinel
