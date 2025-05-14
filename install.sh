#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

echo "Starting Jired installation for Linux/macOS..."

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust cargo is not installed. Please install Rust first:"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Build the release version
echo "Building Jired..."
cargo build --release

# Determine installation directory
INSTALL_DIR=""
if [[ -d "$HOME/.local/bin" && ":$PATH:" == *":$HOME/.local/bin:"* ]]; then
    INSTALL_DIR="$HOME/.local/bin"
elif [[ -d "/usr/local/bin" && -w "/usr/local/bin" ]]; then
    INSTALL_DIR="/usr/local/bin"
else
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
    echo "Created directory $INSTALL_DIR"
    
    # Check if PATH includes the installation directory
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo "Adding $INSTALL_DIR to your PATH..."
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zshrc" 2>/dev/null || true
        echo "Please restart your terminal or run 'source ~/.bashrc' to update your PATH."
    fi
fi

# Copy the binary
echo "Installing Jired to $INSTALL_DIR..."
cp "target/release/jired" "$INSTALL_DIR/"

# Make the binary executable
chmod +x "$INSTALL_DIR/jired"

echo "Jired installation complete! You can now run 'jired' from your terminal."
