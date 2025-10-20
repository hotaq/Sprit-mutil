#!/usr/bin/env python3
"""
Sprite CLI Main Entry Point

This module provides a Python wrapper for the Sprite multi-agent toolkit.
It handles installation, updates, and provides a convenient entry point.
"""

import os
import sys
import subprocess
import platform
import tempfile
import urllib.request
import shutil
from pathlib import Path

SPRITE_VERSION = "0.1.0"
SPRITE_REPO = "https://github.com/hotaq/Sprit-mutil.git"
INSTALL_SCRIPT_URL = "https://raw.githubusercontent.com/hotaq/Sprit-mutil/main/scripts/install.sh"

def get_sprite_binary_path():
    """Get the path to the sprite binary."""
    # Common installation paths
    paths = [
        Path.home() / ".cargo" / "bin" / "sprite",
        Path.home() / ".local" / "bin" / "sprite",
        Path("/usr/local/bin") / "sprite",
        Path("/usr/bin") / "sprite",
    ]

    # Check if sprite exists in PATH
    sprite_in_path = shutil.which("sprite")
    if sprite_in_path:
        return Path(sprite_in_path)

    # Check common installation paths
    for path in paths:
        if path.exists():
            return path

    return None

def ensure_sprite_installed():
    """Ensure sprite is installed, install if necessary."""
    sprite_path = get_sprite_binary_path()

    if sprite_path and sprite_path.exists():
        try:
            # Test if sprite works
            result = subprocess.run([str(sprite_path), "--version"],
                                  capture_output=True, text=True, timeout=10)
            if result.returncode == 0:
                return sprite_path
        except (subprocess.TimeoutExpired, subprocess.SubprocessError, FileNotFoundError):
            pass

    # Sprite not found or not working, install it
    print("🚀 Installing Sprite Multi-Agent Toolkit...")
    return install_sprite()

def install_sprite():
    """Install sprite using the installation script."""
    try:
        # Download installation script
        with tempfile.NamedTemporaryFile(mode='w', suffix='.sh', delete=False) as f:
            print("📥 Downloading installation script...")

            # Use Python to download the script
            import urllib.request
            with urllib.request.urlopen(INSTALL_SCRIPT_URL) as response:
                script_content = response.read().decode('utf-8')
            f.write(script_content)
            script_path = f.name

        # Make script executable
        os.chmod(script_path, 0o755)

        # Run installation script
        print("🔧 Running installation...")
        result = subprocess.run([script_path], capture_output=True, text=True)

        # Cleanup
        os.unlink(script_path)

        if result.returncode == 0:
            print("✅ Sprite installed successfully!")

            # Try to find the installed binary
            import time
            time.sleep(2)  # Give it a moment to complete

            sprite_path = get_sprite_binary_path()
            if sprite_path:
                return sprite_path
            else:
                print("⚠️  Installation completed but sprite binary not found in PATH")
                print("Please restart your terminal or add ~/.cargo/bin to your PATH")
                return None
        else:
            print("❌ Installation failed:")
            print(result.stderr)
            return None

    except Exception as e:
        print(f"❌ Installation failed: {e}")
        return None

def run_sprite(args):
    """Run sprite with the given arguments."""
    sprite_path = ensure_sprite_installed()

    if not sprite_path:
        print("❌ Failed to install or find sprite")
        return 1

    try:
        # Run sprite with the provided arguments
        result = subprocess.run([str(sprite_path)] + args)
        return result.returncode
    except KeyboardInterrupt:
        print("\n👋 Interrupted by user")
        return 130
    except Exception as e:
        print(f"❌ Error running sprite: {e}")
        return 1

def main():
    """Main entry point for the sprite CLI."""
    # Handle version flag
    if len(sys.argv) > 1 and sys.argv[1] in ["--version", "-V"]:
        print(f"sprite {SPRITE_VERSION}")
        return 0

    # Handle help flag
    if len(sys.argv) > 1 and sys.argv[1] in ["--help", "-h"]:
        print("""Sprite Multi-Agent Workflow Toolkit 🤖

USAGE:
    sprite [COMMAND] [OPTIONS]

COMMANDS:
    init          Initialize a new multi-agent environment
    start         Start a multi-agent session
    attach        Attach to an existing session
    kill          Terminate a session
    status        Show system and session status
    agents        Manage AI agents
    config        Configuration management
    help          Show this help message

QUICK START:
    sprite init --agents 3
    sprite start
    sprite attach sprite-session

INSTALLATION:
    This Python wrapper will automatically install the Rust binary if needed.
    For manual installation, visit: https://github.com/hotaq/Sprit-mutil

For detailed help, run: sprite help
""")
        return 0

    # Run sprite with arguments
    return run_sprite(sys.argv[1:])

if __name__ == "__main__":
    sys.exit(main())