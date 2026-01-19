#!/bin/bash
set -e

echo "=== Sierra Launcher - Wayland Only ==="
echo ""

# Check Wayland
if [ -z "$WAYLAND_DISPLAY" ] && [ "$XDG_SESSION_TYPE" != "wayland" ]; then
    echo "ERROR: This launcher requires Wayland"
    echo "Current session: ${XDG_SESSION_TYPE:-unknown}"
    exit 1
fi

# Detect package manager and install deps
echo "Installing dependencies..."
if command -v pacman &>/dev/null; then
    sudo pacman -S --needed rust gcc gtk3 pkg-config brightnessctl pulseaudio redshift
elif command -v apt &>/dev/null; then
    sudo apt update && sudo apt install -y build-essential cargo libgtk-3-dev pkg-config brightnessctl pulseaudio redshift
elif command -v dnf &>/dev/null; then
    sudo dnf install -y rust cargo gtk3-devel gcc brightnessctl pulseaudio redshift
else
    echo "ERROR: Unsupported package manager"
    echo "Please install manually: rust, cargo, gtk3-dev, brightnessctl, pulseaudio, redshift"
    exit 1
fi

# Build
echo ""
echo "Building..."
cargo build --release

# Install
echo "Installing to /usr/local/bin..."
sudo install -m 755 target/release/sierra_launcher /usr/local/bin/sierra-launcher

# Setup config
mkdir -p ~/.config/sierra ~/.cache/sierra

# Generate default config if it doesn't exist
if [ ! -f ~/.config/sierra/Sierra ]; then
    echo "Creating default config..."
    cat > ~/.config/sierra/Sierra << 'EOF'
# Sierra Launcher Configuration

# Font settings
font = "Monocraft"
font_size = 14.0

# Title settings
title_text = " sierra-launcher "
# Available animations: Rainbow, Wave, InOutWave, Pulse, Sparkle, Gradient
title_animation = "Wave"

# Theme settings
# Set to true to use pywal colors from ~/.cache/wal/colors.json
# Set to false to use custom theme below
use_pywal = false

# Custom theme colors (hex format)
# Only used when use_pywal = false
[theme]
background = "#1a1b26"   # Dark background
foreground = "#c0caf5"   # Light foreground text
border = "#7aa2f7"       # Border color
accent = "#7dcfff"       # Accent color

# Terminal color palette
color0 = "#15161e"       # Black
color1 = "#f7768e"       # Red
color2 = "#9ece6a"       # Green
color3 = "#e0af68"       # Yellow
color4 = "#7aa2f7"       # Blue
color5 = "#bb9af7"       # Magenta
color6 = "#7dcfff"       # Cyan
color7 = "#a9b1d6"       # White
color8 = "#414868"       # Bright Black
color9 = "#f7768e"       # Bright Red
color10 = "#9ece6a"      # Bright Green
color11 = "#e0af68"      # Bright Yellow
color12 = "#7aa2f7"      # Bright Blue
color13 = "#bb9af7"      # Bright Magenta
color14 = "#7dcfff"      # Bright Cyan
color15 = "#c0caf5"      # Bright White
EOF
    echo " Config created at ~/.config/sierra/Sierra"
else
    echo " Config already exists at ~/.config/sierra/Sierra"
fi
