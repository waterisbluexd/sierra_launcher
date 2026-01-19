#!/bin/bash

set -e

echo "=== Sierra Launcher Installation ==="
echo ""

# Check if running on Wayland
if [ -z "$WAYLAND_DISPLAY" ] && [ "$XDG_SESSION_TYPE" != "wayland" ]; then
    echo "Warning: This launcher is designed for Wayland. Current session: $XDG_SESSION_TYPE"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Install dependencies
echo "Installing dependencies..."
if command -v pacman &> /dev/null; then
    # Arch Linux
    sudo pacman -S --needed rust gcc gtk3 pkg-config brightnessctl pulseaudio redshift
elif command -v apt &> /dev/null; then
    # Debian/Ubuntu
    sudo apt update
    sudo apt install -y build-essential cargo libgtk-3-dev pkg-config brightnessctl pulseaudio redshift
elif command -v dnf &> /dev/null; then
    # Fedora
    sudo dnf install -y rust cargo gtk3-devel gcc brightnessctl pulseaudio redshift
else
    echo "Unsupported package manager. Please install dependencies manually:"
    echo "  - Rust and Cargo"
    echo "  - GTK3 development files"
    echo "  - brightnessctl, pulseaudio, redshift"
    exit 1
fi

# Build the project
echo ""
echo "Building sierra-launcher..."
cargo build --release

# Install binary
echo ""
echo "Installing binary..."
sudo cp target/release/sierra_launcher /usr/local/bin/sierra-launcher
sudo chmod +x /usr/local/bin/sierra-launcher

# Create config directory
echo "Creating config directory..."
mkdir -p ~/.config/sierra

# Copy config file
if [ -f "config/Sierra" ]; then
    cp config/Sierra ~/.config/sierra/Sierra
    echo "Config file copied to ~/.config/sierra/Sierra"
fi

# Create cache directory
mkdir -p ~/.cache/sierra

echo ""
echo "=== Installation Complete! ==="
echo ""
echo "To bind to Super+F, you need to add a keybinding in your window manager."
echo ""
echo "For Hyprland, add this to ~/.config/hypr/hyprland.conf:"
echo "  bind = SUPER, F, exec, sierra-launcher"
echo ""
echo "For Sway, add this to ~/.config/sway/config:"
echo "  bindsym Mod4+f exec sierra-launcher"
echo ""
echo "For KDE Plasma (Wayland), go to:"
echo "  System Settings > Shortcuts > Custom Shortcuts"
echo "  Add new command shortcut with 'sierra-launcher'"
echo ""
echo "For GNOME (Wayland), run:"
echo "  gsettings set org.gnome.settings-daemon.plugins.media-keys custom-keybindings \"['/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/']\""
echo "  gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/ name 'Sierra Launcher'"
echo "  gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/ command 'sierra-launcher'"
echo "  gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/ binding '<Super>f'"
echo ""

# Detect window manager and offer to configure
if pgrep -x "Hyprland" > /dev/null; then
    echo "Hyprland detected!"
    read -p "Would you like to automatically add Super+F binding to hyprland.conf? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        HYPR_CONF="$HOME/.config/hypr/hyprland.conf"
        if [ -f "$HYPR_CONF" ]; then
            if ! grep -q "sierra-launcher" "$HYPR_CONF"; then
                echo "" >> "$HYPR_CONF"
                echo "# Sierra Launcher" >> "$HYPR_CONF"
                echo "bind = SUPER, F, exec, sierra-launcher" >> "$HYPR_CONF"
                echo "✓ Added keybinding to $HYPR_CONF"
                echo "  Reload Hyprland config to apply changes"
            else
                echo "⚠ Keybinding already exists in $HYPR_CONF"
            fi
        fi
    fi
elif pgrep -x "sway" > /dev/null; then
    echo "Sway detected!"
    read -p "Would you like to automatically add Super+F binding to sway config? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        SWAY_CONF="$HOME/.config/sway/config"
        if [ -f "$SWAY_CONF" ]; then
            if ! grep -q "sierra-launcher" "$SWAY_CONF"; then
                echo "" >> "$SWAY_CONF"
                echo "# Sierra Launcher" >> "$SWAY_CONF"
                echo "bindsym Mod4+f exec sierra-launcher" >> "$SWAY_CONF"
                echo "✓ Added keybinding to $SWAY_CONF"
                echo "  Reload sway config to apply changes (Mod+Shift+C)"
            else
                echo "⚠ Keybinding already exists in $SWAY_CONF"
            fi
        fi
    fi
fi

echo ""
echo "You can now run 'sierra-launcher' or press Super+F (if configured)"