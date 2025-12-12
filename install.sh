#!/bin/bash
set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}🛡️  PortSentinel Installer 🛡️${NC}"

if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}This script must be run as root.${NC}" 
   exit 1
fi

INSTALL_MASTER=false
INSTALL_AGENT=false

# Interactive or Flag Mode
if [[ "$1" == "--master" ]]; then
    INSTALL_MASTER=true
    INSTALL_AGENT=true # Master always needs a local agent for self-monitoring
elif [[ "$1" == "--agent" ]]; then
    INSTALL_AGENT=true
else
    echo "Select Installation Type:"
    echo "1) Master Server (Installs Dashboard + Agent)"
    echo "2) Remote Node (Installs Agent Only)"
    read -p "Enter choice [1/2]: " choice
    if [[ "$choice" == "1" ]]; then
        INSTALL_MASTER=true
        INSTALL_AGENT=true
    else
        INSTALL_AGENT=true
        read -p "Enter Agent Authentication Token: " AGENT_TOKEN
    fi
fi

# directories
BIN_DIR="/usr/local/bin"
ASSETS_DIR="/var/lib/port_sentinel"
CONFIG_DIR="/etc/port_sentinel"

mkdir -p $ASSETS_DIR
mkdir -p $CONFIG_DIR

# --- AGENT INSTALLATION ---
if [ "$INSTALL_AGENT" = true ]; then
    echo -e "${GREEN}Installing Agent...${NC}"
    cp port_sentinel_agent $BIN_DIR/
    chmod +x $BIN_DIR/port_sentinel_agent

    # Create systemd service for Agent
    cat <<EOF > /etc/systemd/system/port-sentinel-agent.service
[Unit]
Description=PortSentinel Agent
After=network.target

[Service]
ExecStart=$BIN_DIR/port_sentinel_agent 
Restart=always
User=root
WorkingDirectory=$ASSETS_DIR



[Install]
WantedBy=multi-user.target
EOF

    # Create default config if missing
    if [ ! -f "$ASSETS_DIR/config.json" ]; then
        echo "creating default config.json"
        cat <<CJ > "$ASSETS_DIR/config.json"
{
  "port": 3001,
  "hostname": "localhost",
  "auth_token": "${AGENT_TOKEN:-change_me_please}"
}
CJ
        chmod 600 "$ASSETS_DIR/config.json"
    fi

    systemctl daemon-reload
    systemctl enable --now port-sentinel-agent
    echo -e "${GREEN}✅ Agent installed and started on port 3001.${NC}"
fi

# --- MASTER INSTALLATION ---
if [ "$INSTALL_MASTER" = true ]; then
    echo -e "${GREEN}Installing Master Dashboard...${NC}"
    cp port_sentinel_master $BIN_DIR/
    chmod +x $BIN_DIR/port_sentinel_master
    
    # Copy Assets
    echo "Copying assets..."
    cp -r assets $ASSETS_DIR/

    # Create systemd service for Master
    cat <<EOF > /etc/systemd/system/port-sentinel-master.service
[Unit]
Description=PortSentinel Master Dashboard
After=network.target port-sentinel-agent.service

[Service]
ExecStart=$BIN_DIR/port_sentinel_master
Restart=always
User=root
WorkingDirectory=$ASSETS_DIR

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    systemctl enable --now port-sentinel-master
    echo -e "${GREEN}✅ Master Dashboard installed and started on port 7878.${NC}"
    echo -e "${BLUE}👉 Access dashboard at http://<YOUR_IP>:7878${NC}"
fi

echo -e "${GREEN}🎉 Installation Complete!${NC}"
