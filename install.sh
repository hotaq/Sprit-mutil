#!/bin/bash

# Sprite Installation Script
# Installs Sprite multi-agent toolkit from various sources

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_DIR="${HOME}/.local/bin"
CARGO_INSTALLED=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."

    # Check for Rust/Cargo
    if command_exists cargo; then
        CARGO_INSTALLED=true
        print_success "Cargo found"
    else
        print_warning "Cargo not found. Will install pre-compiled binary if available."
    fi

    # Check for git
    if command_exists git; then
        print_success "Git found"
    else
        print_error "Git is required but not installed. Please install Git first."
        exit 1
    fi

    # Check for tmux
    if command_exists tmux; then
        print_success "Tmux found"
    else
        print_error "Tmux is required but not installed. Please install Tmux first."
        exit 1
    fi
}

# Install using cargo (if available)
install_with_cargo() {
    local source="$1"

    if [ "$CARGO_INSTALLED" = true ]; then
        print_status "Installing Sprite using Cargo from $source..."

        case "$source" in
            "crates")
                cargo install sprite
                ;;
            "github")
                cargo install --git https://github.com/hotaq/Sprit-mutil.git sprite
                ;;
            "local")
                if [ -f "$SCRIPT_DIR/Cargo.toml" ]; then
                    cargo install --path "$SCRIPT_DIR"
                else
                    print_error "Local Cargo.toml not found. Cannot install from source."
                    return 1
                fi
                ;;
        esac

        if [ $? -eq 0 ]; then
            print_success "Sprite installed successfully via Cargo!"
            return 0
        else
            print_error "Cargo installation failed."
            return 1
        fi
    else
        print_warning "Cargo not available. Cannot install using this method."
        return 1
    fi
}

# Install pre-compiled binary
install_binary() {
    print_status "Installing pre-compiled binary..."

    # Detect platform
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)

    case "$arch" in
        x86_64)
            arch="x86_64"
            ;;
        aarch64|arm64)
            arch="aarch64"
            ;;
        *)
            print_error "Unsupported architecture: $arch"
            return 1
            ;;
    esac

    case "$os" in
        linux)
            os="unknown-linux-gnu"
            ;;
        darwin)
            os="apple-darwin"
            ;;
        *)
            print_error "Unsupported OS: $os"
            return 1
            ;;
    esac

    local filename="sprite-${arch}-${os}.tar.gz"
    local download_url="https://github.com/hotaq/Sprit-mutil/releases/latest/download/${filename}"

    print_status "Downloading from: $download_url"

    # Create temp directory
    local temp_dir=$(mktemp -d)
    cd "$temp_dir"

    # Download and extract
    if curl -L -o "$filename" "$download_url" && tar -xzf "$filename"; then
        # Ensure install directory exists
        mkdir -p "$INSTALL_DIR"

        # Move binary to install directory
        if mv sprite "$INSTALL_DIR/"; then
            chmod +x "$INSTALL_DIR/sprite"
            cd - && rm -rf "$temp_dir"
            print_success "Binary installed to $INSTALL_DIR/sprite"
            return 0
        else
            cd - && rm -rf "$temp_dir"
            print_error "Failed to move binary to $INSTALL_DIR"
            return 1
        fi
    else
        cd - && rm -rf "$temp_dir"
        print_error "Failed to download or extract binary"
        return 1
    fi
}

# Main installation logic
main() {
    echo "🚀 Sprite Multi-Agent Toolkit Installation Script"
    echo "=================================================="
    echo

    # Parse arguments
    INSTALL_METHOD="auto"
    while [[ $# -gt 0 ]]; do
        case $1 in
            --method)
                INSTALL_METHOD="$2"
                shift 2
                ;;
            --crates)
                INSTALL_METHOD="crates"
                shift
                ;;
            --github)
                INSTALL_METHOD="github"
                shift
                ;;
            --binary)
                INSTALL_METHOD="binary"
                shift
                ;;
            --local)
                INSTALL_METHOD="local"
                shift
                ;;
            --help|-h)
                echo "Usage: $0 [OPTIONS]"
                echo
                echo "Options:"
                echo "  --method METHOD    Installation method: auto, crates, github, binary, local"
                echo "  --crates           Install from crates.io using cargo"
                echo "  --github           Install from GitHub using cargo"
                echo "  --binary           Install pre-compiled binary"
                echo "  --local            Install from local source"
                echo "  --help, -h         Show this help message"
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done

    # Check prerequisites
    check_prerequisites
    echo

    # Install based on method
    case "$INSTALL_METHOD" in
        "crates")
            install_with_cargo "crates"
            ;;
        "github")
            install_with_cargo "github"
            ;;
        "binary")
            install_binary
            ;;
        "local")
            install_with_cargo "local"
            ;;
        "auto")
            # Try different methods in order of preference
            if install_with_cargo "crates"; then
                print_success "Installation completed via crates.io!"
            elif install_with_cargo "github"; then
                print_success "Installation completed via GitHub!"
            elif install_binary; then
                print_success "Installation completed via binary!"
            else
                print_error "All installation methods failed."
                exit 1
            fi
            ;;
        *)
            print_error "Unknown installation method: $INSTALL_METHOD"
            exit 1
            ;;
    esac

    echo
    print_success "Sprite installation completed!"
    echo

    # Verify installation
    if command_exists sprite; then
        print_status "Sprite version: $(sprite --version)"
        echo
        print_status "To get started:"
        echo "  1. Navigate to your git repository: cd /path/to/your/project"
        echo "  2. Initialize Sprite: sprite init"
        echo "  3. Start a session: sprite start"
        echo "  4. For help: sprite --help"
    else
        print_warning "Sprite command not found in PATH. You may need to:"
        echo "  • Add $INSTALL_DIR to your PATH"
        echo "  • Restart your terminal"
        echo "  • Or run: export PATH=\"\$PATH:$INSTALL_DIR\""
    fi
}

# Run main function
main "$@"