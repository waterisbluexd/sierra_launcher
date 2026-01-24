#!/bin/bash
set -e

RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m'

echo -e "${YELLOW}=== Sierra Launcher (Wayland Only) ===${NC}"
echo ""

# ──────────────────────────────────────────────
# Wayland check
# ──────────────────────────────────────────────
if [ -z "$WAYLAND_DISPLAY" ] && [ "$XDG_SESSION_TYPE" != "wayland" ]; then
    echo -e "${RED}ERROR: Sierra Launcher requires Wayland${NC}"
    echo "Current session: ${XDG_SESSION_TYPE:-unknown}"
    exit 1
fi

# ──────────────────────────────────────────────
# Install dependencies
# ──────────────────────────────────────────────
echo -e "${YELLOW}Installing dependencies...${NC}"

if command -v pacman &>/dev/null; then
    sudo pacman -S --needed \
        rust cargo gcc pkg-config gtk3 \
        brightnessctl pulseaudio redshift ffmpeg \
        lm_sensors
    
    if ! command -v gslapper &>/dev/null; then
        echo -e "${YELLOW}gSlapper not found, installing from AUR...${NC}"
        if command -v yay &>/dev/null; then
            yay -S --needed gslapper
        elif command -v paru &>/dev/null; then
            paru -S --needed gslapper
        else
            echo -e "${YELLOW}No AUR helper found (yay/paru)${NC}"
            echo -e "${YELLOW}Install gSlapper manually:${NC}"
            echo "  yay -S gslapper"
            echo "  OR from: https://gitlab.com/phoneybadger/gslapper"
        fi
    fi

elif command -v apt &>/dev/null; then
    sudo apt update
    sudo apt install -y \
        build-essential cargo pkg-config libgtk-3-dev \
        brightnessctl pulseaudio redshift ffmpeg \
        lm-sensors

    if ! command -v gslapper &>/dev/null; then
        echo -e "${YELLOW}gSlapper not found in apt, install manually from:${NC}"
        echo "https://gitlab.com/phoneybadger/gslapper"
    fi

elif command -v dnf &>/dev/null; then
    sudo dnf install -y \
        rust cargo gcc pkg-config gtk3-devel \
        brightnessctl pulseaudio redshift ffmpeg \
        lm_sensors

    if ! command -v gslapper &>/dev/null; then
        echo -e "${YELLOW}gSlapper not found in dnf, install manually from:${NC}"
        echo "https://gitlab.com/phoneybadger/gslapper"
    fi

else
    echo -e "${RED}ERROR: Unsupported package manager${NC}"
    echo "Please install manually:"
    echo "  rust, cargo, gtk3-dev, pkg-config, brightnessctl, pulseaudio"
    echo "  redshift, ffmpeg, gslapper, lm_sensors"
    exit 1
fi

# ──────────────────────────────────────────────
# Build
# ──────────────────────────────────────────────
echo ""
echo -e "${YELLOW}Building Sierra Launcher (release)...${NC}"
cargo build --release

# ──────────────────────────────────────────────
# Install binary
# ──────────────────────────────────────────────
echo -e "${YELLOW}Installing binary...${NC}"
sudo install -Dm755 target/release/sierra_launcher /usr/local/bin/sierra-launcher

# ──────────────────────────────────────────────
# Config & cache directories
# ──────────────────────────────────────────────
CONFIG_DIR="$HOME/.config/sierra"
CACHE_DIR="$HOME/.cache/sierra"
CONFIG_FILE="$CONFIG_DIR/Sierra"

mkdir -p "$CONFIG_DIR"
mkdir -p "$CACHE_DIR"

# ──────────────────────────────────────────────
# Default wallpaper directory
# ──────────────────────────────────────────────
DEFAULT_WALLPAPER_DIR="$HOME/Pictures/Wallpapers"
mkdir -p "$DEFAULT_WALLPAPER_DIR"

# ──────────────────────────────────────────────
# Generate default config
# ──────────────────────────────────────────────
if [ ! -f "$CONFIG_FILE" ]; then
    echo -e "${YELLOW}Creating default config...${NC}"

    cat > "$CONFIG_FILE" << EOF
font = "Monospace"
font_size = 14.0

title_text = " sierra-launcher "
title_animation = "Wave"

wallpaper_dir = "$DEFAULT_WALLPAPER_DIR"

use_pywal = false

[theme]
background = "#1a1b26"
foreground = "#c0caf5"
border     = "#7aa2f7"
accent     = "#7dcfff"

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

    echo -e "${GREEN}✓ Config created at $CONFIG_FILE${NC}"
else
    echo -e "${GREEN}✓ Config already exists at $CONFIG_FILE${NC}"
fi

echo ""
echo -e "${GREEN}✓ Sierra Launcher installed successfully${NC}"
echo -e "${GREEN}→ Binary: /usr/local/bin/sierra-launcher${NC}"
echo -e "${GREEN}→ Config: $CONFIG_FILE${NC}"
echo -e "${GREEN}→ Cache:  $CACHE_DIR${NC}"
echo ""
echo -e "${YELLOW}Note: Add wallpapers to $DEFAULT_WALLPAPER_DIR${NC}"
echo -e "${YELLOW}      Supported formats: jpg, png, webp, mp4, mkv, webm${NC}"