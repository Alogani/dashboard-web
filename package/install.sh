#!/bin/sh

# Exit on error
set -e

# Default paths
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="/etc/home-webserver"
INIT_SCRIPT="/etc/init.d/home-webserver"

# Get the script directory
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Check if running as root
if [ "$(id -u)" -ne 0 ]; then
    echo "This script must be run as root" >&2
    exit 1
fi

# Create directories
echo "Creating configuration directory..."
mkdir -p "$CONFIG_DIR"

# Install binary
echo "Installing home-webserver binary..."
if [ -f "$SCRIPT_DIR/home-webserver" ]; then
    cp "$SCRIPT_DIR/home-webserver" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/home-webserver"
else
    echo "Error: Binary not found at $SCRIPT_DIR/home-webserver" >&2
    echo "Please run build.sh first" >&2
    exit 1
fi

# Install configuration
echo "Installing configuration files..."
if [ ! -f "$CONFIG_DIR/config.toml" ]; then
    cp "$SCRIPT_DIR/config.toml" "$CONFIG_DIR/"
else
    echo "Config file already exists, not overwriting"
    echo "To use the new config, manually copy from $SCRIPT_DIR/config.toml"
fi

# Create empty users file if it doesn't exist
if [ ! -f "$CONFIG_DIR/users.txt" ]; then
    echo "Creating empty users.txt file..."
    touch "$CONFIG_DIR/users.txt"
fi

# Install init script
echo "Installing OpenRC init script..."
cp "$SCRIPT_DIR/home-webserver-init-script" "$INIT_SCRIPT"
chmod +x "$INIT_SCRIPT"

echo "Installation complete!"
echo ""
echo "To enable the service at boot:"
echo "  rc-update add home-webserver default"
echo ""
echo "To start the service now:"
echo "  rc-service home-webserver start"
echo ""
echo "Configuration is located at $CONFIG_DIR/config.toml"
echo "Users file is located at $CONFIG_DIR/users.txt"