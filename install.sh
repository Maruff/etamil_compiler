#!/bin/bash
# eTamil Compiler Installation Script
# This script installs the eTamil compiler system-wide

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  eTamil Compiler Installation Script  ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

# Check if binary exists
BINARY_PATH="./etamil_compiler/target/release/etamil_compiler"
if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${RED}✗ Error: eTamil compiler binary not found at $BINARY_PATH${NC}"
    echo -e "${BLUE}Building the compiler...${NC}"
    cd etamil_compiler
    cargo build --release
    cd ..
    if [ ! -f "$BINARY_PATH" ]; then
        echo -e "${RED}✗ Build failed. Please check for errors.${NC}"
        exit 1
    fi
fi

# Get installation directory
if [ "$EUID" -eq 0 ]; then
    # Root installation
    INSTALL_DIR="/usr/local/bin"
    INSTALL_NAME="etamil"
else
    # User installation
    INSTALL_DIR="$HOME/.local/bin"
    INSTALL_NAME="etamil"
    mkdir -p "$INSTALL_DIR"
fi

echo -e "${BLUE}Installation directory: $INSTALL_DIR${NC}"

# Copy binary
echo -e "${BLUE}Installing eTamil compiler...${NC}"
cp "$BINARY_PATH" "$INSTALL_DIR/$INSTALL_NAME"
chmod +x "$INSTALL_DIR/$INSTALL_NAME"

# Verify installation
if [ -f "$INSTALL_DIR/$INSTALL_NAME" ]; then
    echo -e "${GREEN}✓ eTamil compiler installed successfully!${NC}"
    echo ""
    
    # Check if directory is in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo -e "${BLUE}ℹ Note: $INSTALL_DIR is not in your PATH${NC}"
        echo -e "${BLUE}Add this line to your ~/.bashrc or ~/.zshrc:${NC}"
        echo -e "${GREEN}export PATH=\"\$PATH:$INSTALL_DIR\"${NC}"
        echo ""
    fi
    
    # Show version and usage
    echo -e "${BLUE}Installation complete!${NC}"
    echo ""
    echo -e "${GREEN}Usage:${NC}"
    echo -e "  ${BLUE}etamil compile <source.etamil> -o <output.qmz>${NC}  # Compile eTamil code"
    echo -e "  ${BLUE}etamil run <program.qmz>${NC}                      # Run compiled program"
    echo -e "  ${BLUE}etamil server <port> <etamil_file>${NC}           # Start HTTP server"
    echo -e "  ${BLUE}etamil --help${NC}                                # Show all commands"
    echo ""
    echo -e "${GREEN}Examples:${NC}"
    echo -e "  ${BLUE}etamil compile hello.etamil -o hello.qmz${NC}"
    echo -e "  ${BLUE}etamil run hello.qmz${NC}"
    echo -e "  ${BLUE}etamil server 8080 examples/backend/crud.etamil${NC}"
    echo ""
    
    # Show binary info
    BINARY_SIZE=$(du -h "$INSTALL_DIR/$INSTALL_NAME" | cut -f1)
    echo -e "${BLUE}Binary size: $BINARY_SIZE${NC}"
    echo -e "${BLUE}Installed at: $INSTALL_DIR/$INSTALL_NAME${NC}"
else
    echo -e "${RED}✗ Installation failed${NC}"
    exit 1
fi
