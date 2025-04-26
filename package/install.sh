#!/bin/sh

# Exit on error
set -e

# Default paths
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="/etc/dashboard-web"
INIT_SCRIPT="/etc/init.d/dashboard-web"
STATIC_DIR="$CONFIG_DIR/static"

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
mkdir -p "$STATIC_DIR"

# Install binary
echo "Installing dashboard-web binary..."
if [ -f "$SCRIPT_DIR/dashboard-web" ]; then
    cp "$SCRIPT_DIR/dashboard-web" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/dashboard-web"
else
    echo "Error: Binary not found at $SCRIPT_DIR/dashboard-web" >&2
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
if [ ! -f "$CONFIG_DIR/users_db.txt" ]; then
    echo "Creating empty users_db.txt file..."
    touch "$CONFIG_DIR/users_db.txt"
fi

# Install static files
echo "Installing static files..."
if [ -d "$SCRIPT_DIR/static" ]; then
    cp -r "$SCRIPT_DIR/static/"* "$STATIC_DIR/"
    echo "Static files installed to $STATIC_DIR"
else
    echo "Warning: Static directory not found at $SCRIPT_DIR/static" >&2
    echo "Static files were not installed" >&2
fi

# Install init script
echo "Installing OpenRC init script..."
cp "$SCRIPT_DIR/openrc-init" "$INIT_SCRIPT"
chmod +x "$INIT_SCRIPT"

echo "Installation complete!"
echo ""
echo "To enable the service at boot:"
echo "  rc-update add dashboard-web default"
echo ""
echo "To start the service now:"
echo "  rc-service dashboard-web start"
echo ""
echo "Configuration is located at $CONFIG_DIR/config.toml"
echo "Static files are located at $STATIC_DIR"
