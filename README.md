# PortSentinel 🛡️

PortSentinel is a centralized system monitoring and management tool written in Rust. It follows a Master-Agent architecture to monitor multiple Linux servers from a single dashboard.

## 🚀 Features

-   **Real-time Monitoring**: CPU, Memory, Disk, and Swap usage.
-   **Process Management**: View active processes, kill processes, and inspect open files/ports (via `lsof`).
-   **Log Inspector**: Remotely view log files associated with running processes.
-   **Service Manager**: Control systemd services (`systemctl status`, `start`, `stop`, `restart`) from the dashboard.
-   **Multi-Node Architecture**: Connect multiple Agents to a single Master Dashboard.

## 🏗️ Architecture

1.  **Agent**: A lightweight binary running on Linux servers. It gathers system stats and exposes a REST API.
    *   *Requires `sudo`* for `lsof` and `systemctl` access.
2.  **Master**: The central dashboard server. It aggregates data from Agents and renders the UI (Askama + HTMX).

## 🛠️ Getting Started

### Prerequisites
-   Rust (latest stable)
-   Linux / macOS (for Agent)

### 1. Running the Agent (Target Node)
The agent must run on the server you want to monitor.

```bash
# Run with sudo to enable full features (Process/Service control)
sudo cargo run -p port_sentinel_agent
```
By default, it listens on port `3001`.

### 2. Running the Master (Dashboard)
The master hosts the user interface.

```bash
cargo run -p port_sentinel_master
```
Access the dashboard at [http://localhost:7878](http://localhost:7878).

### 3. Adding Nodes
1.  Go to the Dashboard.
2.  Click **+ Add Node** in the sidebar.
3.  Enter the Agent URL (e.g., `http://192.168.1.50:3001`) and Auth Token (if set).

## 🔒 Security Note
Since the Agent runs validation commands (`systemctl`, `kill`), it implements strict input sanitization:
-   **Service Names**: Alphanumeric, dots, and hyphens only.
-   **Auth**: Supports Token-based authentication (configure in `config.json`).

## 🐳 Docker Support (Coming Soon)
We are currently working on full Docker support to allow:
-   Monitoring/Controlling Docker containers on nodes.
-   Running PortSentinel Master/Agent as containers.

## 📄 License
MIT
