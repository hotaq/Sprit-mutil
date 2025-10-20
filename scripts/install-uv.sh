#!/bin/bash

# UV-Style Installation Script for Sprite
# Alternative to 'uv tool install' for Rust tools

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Check for uv
check_uv() {
    if command -v uv >/dev/null 2>&1; then
        print_info "uv found - installing sprite as uv tool"
        return 0
    else
        print_warning "uv not found - falling back to cargo install"
        return 1
    fi
}

# Install using uv
install_with_uv() {
    print_info "Installing sprite with uv..."

    # Use local installation approach
    if uv tool install . --force; then
        print_success "Sprite installed with uv!"
        print_info "Run: sprite --help"
        return 0
    else
        print_warning "uv installation failed, trying cargo install..."
        return 1
    fi
}

# Install using cargo-binstall (binary installation)
install_with_cargo_binstall() {
    if command -v cargo-binstall >/dev/null 2>&1; then
        print_info "Installing sprite with cargo-binstall (fast binary installation)..."
        if cargo-binstall sprite --force; then
            print_success "Sprite installed with cargo-binstall!"
            print_info "Run: sprite --help"
            return 0
        fi
    fi
    return 1
}

# Install using cargo install (from source)
install_with_cargo() {
    if command -v cargo >/dev/null 2>&1; then
        print_info "Installing sprite with cargo install (from source)..."
        if cargo install sprite --force; then
            print_success "Sprite installed with cargo!"
            print_info "Run: sprite --help"
            return 0
        fi
    fi
    return 1
}

# Main installation
main() {
    echo "🚀 Sprite Multi-Agent Toolkit Installation"
    echo "=========================================="
    echo

    # Try uv first (most modern)
    if check_uv; then
        if install_with_uv; then
            exit 0
        fi
    fi

    # Try cargo-binstall (fast binary installation)
    if install_with_cargo_binstall; then
        exit 0
    fi

    # Fall back to cargo install
    if install_with_cargo; then
        exit 0
    fi

    echo "❌ All installation methods failed"
    echo "Please install manually: https://github.com/hotaq/Sprit-mutil"
    exit 1
}

main "$@"