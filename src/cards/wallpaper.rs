use serde_json::Value as JsonValue;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn current_wallpaper_image() -> slint::Image {
    current_wallpaper_path()
        .and_then(|path| slint::Image::load_from_path(&path).ok())
        .unwrap_or_default()
}

fn current_wallpaper_path() -> Option<PathBuf> {
    if let Some(wallpaper) = env::var_os("SIERRA_LAUNCHER_WALLPAPER") {
        let path = PathBuf::from(wallpaper);
        if path.exists() && is_supported_image(&path) {
            return Some(path);
        }
    }

    pywal_wallpaper().or_else(wallpapers_dir_wallpaper).or_else(gnome_wallpaper).or_else(kde_wallpaper).or_else(xfce_wallpaper)
}

fn pywal_wallpaper() -> Option<PathBuf> {
    let home = env::var_os("HOME")?;
    let cache_dir = PathBuf::from(home).join(".cache/wal");

    if let Some(path) = pywal_colors_json(&cache_dir) {
        return Some(path);
    }

    if let Some(path) = pywal_wal_file(&cache_dir) {
        return Some(path);
    }

    None
}

fn pywal_colors_json(cache_dir: &Path) -> Option<PathBuf> {
    let config_path = cache_dir.join("colors.json");
    let contents = fs::read_to_string(&config_path).ok()?;
    let json: JsonValue = serde_json::from_str(&contents).ok()?;
    let wallpaper = json.get("wallpaper")?.as_str()?;
    let path = PathBuf::from(wallpaper);

    if path.exists() && is_supported_image(&path) {
        return Some(path);
    }

    wallpapers_dir_match(&path)
}

fn pywal_wal_file(cache_dir: &Path) -> Option<PathBuf> {
    let wal_file = cache_dir.join("wal");
    let contents = fs::read_to_string(&wal_file).ok()?;
    let whitespace = contents.lines().next()?.trim();
    let path = PathBuf::from(whitespace);

    if path.exists() && is_supported_image(&path) {
        return Some(path);
    }

    wallpapers_dir_match(&path)
}

fn wallpapers_dir_wallpaper() -> Option<PathBuf> {
    let home = env::var_os("HOME")?;
    let wallpapers_dir = PathBuf::from(home).join("Wallpapers");
    let entries = fs::read_dir(&wallpapers_dir).ok()?;

    let mut images: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.is_file() && is_supported_image(path))
        .collect();

    images.sort_by_key(|path| fs::metadata(path).and_then(|m| m.modified()).ok());
    images.pop()
}

fn wallpapers_dir_match(path: &Path) -> Option<PathBuf> {
    let home = env::var_os("HOME")?;
    let wallpapers_dir = PathBuf::from(home).join("Wallpapers");
    let basename = path.file_name()?;

    fs::read_dir(&wallpapers_dir)
        .ok()?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .find(|candidate| candidate.file_name() == Some(basename) && is_supported_image(candidate))
}

fn is_supported_image(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext.to_lowercase().as_str(), "png" | "jpg" | "jpeg" | "webp" | "bmp" | "gif" | "svg"))
        .unwrap_or(false)
}

fn parse_file_uri(value: &str) -> Option<PathBuf> {
    let stripped = value.trim().trim_matches(|c| c == '"' || c == '\'');
    let stripped = stripped.strip_prefix("file://").unwrap_or(stripped);
    let path = PathBuf::from(stripped);
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

fn gnome_wallpaper() -> Option<PathBuf> {
    let output = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.background", "picture-uri"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_file_uri(&stdout)
}

fn kde_wallpaper() -> Option<PathBuf> {
    let home = env::var_os("HOME")?;
    let config_path = PathBuf::from(home).join(".config/plasma-org.kde.plasma.desktop-appletsrc");
    let contents = fs::read_to_string(config_path).ok()?;

    for line in contents.lines() {
        if let Some(start) = line.find("Image=file://") {
            let value = &line[start + "Image=".len()..];
            if let Some(path) = parse_file_uri(value) {
                return Some(path);
            }
        }
    }

    None
}

fn xfce_wallpaper() -> Option<PathBuf> {
    let home = env::var_os("HOME")?;
    let config_path = PathBuf::from(home).join(".config/xfce4/xfconf/xfce4-desktop.xml");
    let contents = fs::read_to_string(config_path).ok()?;

    for line in contents.lines() {
        if line.contains("image-path") || line.contains("last-image") {
            if let Some(start) = line.find("value=\"") {
                let rest = &line[start + 7..];
                if let Some(end) = rest.find('"') {
                    let candidate = &rest[..end];
                    let path = PathBuf::from(candidate);
                    if path.exists() {
                        return Some(path);
                    }
                }
            }
        }
    }

    None
}
