#!/bin/bash
set -e

echo "=== Sierra Launcher (Wayland Only) ==="
echo ""

# ──────────────────────────────
# Wayland check
# ──────────────────────────────
if [ -z "$WAYLAND_DISPLAY" ] && [ "$XDG_SESSION_TYPE" != "wayland" ]; then
    echo "ERROR: Sierra Launcher requires Wayland"
    echo "Current session: ${XDG_SESSION_TYPE:-unknown}"
    exit 1
fi

# ──────────────────────────────
# Install dependencies
# ──────────────────────────────
echo "Installing dependencies..."

if command -v pacman &>/dev/null; then
    sudo pacman -S --needed \
        rust cargo gcc pkg-config gtk3 \
        brightnessctl pulseaudio redshift ffmpeg

elif command -v apt &>/dev/null; then
    sudo apt update
    sudo apt install -y \
        build-essential cargo pkg-config libgtk-3-dev \
        brightnessctl pulseaudio redshift ffmpeg

elif command -v dnf &>/dev/null; then
    sudo dnf install -y \
        rust cargo gcc pkg-config gtk3-devel \
        brightnessctl pulseaudio redshift ffmpeg

else
    echo "ERROR: Unsupported package manager"
    echo "Please install manually:"
    echo "  rust, cargo, gtk3-dev, pkg-config, brightnessctl, pulseaudio, redshift, ffmpeg"
    exit 1
fi

# ──────────────────────────────
# Build
# ──────────────────────────────
echo ""
echo "Building Sierra Launcher (release)..."
cargo build --release

# ──────────────────────────────
# Install binary
# ──────────────────────────────
echo "Installing binary..."
sudo install -Dm755 target/release/sierra_launcher /usr/local/bin/sierra-launcher

# ──────────────────────────────
# Config & cache directories
# ──────────────────────────────
CONFIG_DIR="$HOME/.config/sierra"
CACHE_DIR="$HOME/.cache/sierra"
CONFIG_FILE="$CONFIG_DIR/Sierra"

mkdir -p "$CONFIG_DIR"
mkdir -p "$CACHE_DIR"

# ──────────────────────────────
# Default wallpaper directory
# ──────────────────────────────
DEFAULT_WALLPAPER_DIR="$HOME/Pictures/Wallpapers"

# Create default wallpaper dir if missing (non-fatal)
mkdir -p "$DEFAULT_WALLPAPER_DIR"

# ──────────────────────────────
# Generate default config
# ──────────────────────────────
if [ ! -f "$CONFIG_FILE" ]; then
    echo "Creating default config..."

    cat > "$CONFIG_FILE" << EOF
# ==============================
# Sierra Launcher Configuration
# ==============================

font = "Monospace"
font_size = 14.0

title_text = " sierra-launcher "
# Rainbow, Wave, InOutWave, Pulse, Sparkle, Gradient
title_animation = "Wave"

# Wallpaper directory
# Used by the wallpaper panel & cache system
wallpaper_dir = "$DEFAULT_WALLPAPER_DIR"

# true  → use pywal colors
# false → use custom theme below
use_pywal = false

[theme]
background = "#1a1b26"
foreground = "#c0caf5"
border     = "#7aa2f7"
accent    = "#7dcfff"

color0  = "#15161e"
color1  = "#f7768e"
color2  = "#9ece6a"
color3  = "#e0af68"
color4  = "#7aa2f7"
color5  = "#bb9af7"
color6  = "#7dcfff"
color7  = "#a9b1d6"

color8  = "#414868"
color9  = "#f7768e"
color10 = "#9ece6a"
color11 = "#e0af68"
color12 = "#7aa2f7"
color13 = "#bb9af7"
color14 = "#7dcfff"
color15 = "#c0caf5"
EOF

    echo "Config created at $CONFIG_FILE"
else
    echo "Config already exists at $CONFIG_FILE"
fi

echo ""
echo "✔ Sierra Launcher installed successfully"
echo "→ Binary: /usr/local/bin/sierra-launcher"
echo "→ Config: $CONFIG_FILE"
echo "→ Cache:  $CACHE_DIR"
