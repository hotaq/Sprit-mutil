# Scripts

This directory contains utility and installation scripts for Sprite.

## Files

- **install.sh** - Main installation script (supports multiple methods)
- **test_accessibility.sh** - Accessibility testing suite
- **debug_tmux** - Tmux debugging utility (compiled binary)
- **debug_tmux.rs** - Source for tmux debugging utility

## Usage

### Installation Script
```bash
# Run directly
./install.sh

# Or with specific method
./install.sh --method crates
./install.sh --method binary
./install.sh --local

# Get help
./install.sh --help
```

### Testing
```bash
# Run accessibility tests
./test_accessibility.sh

# Debug tmux sessions
./debug_tmux --help
```

## Installation from Web

The installation script can also be run directly from the web:
```bash
curl -fsSL https://raw.githubusercontent.com/hotaq/Sprit-mutil/main/scripts/install.sh | bash
```