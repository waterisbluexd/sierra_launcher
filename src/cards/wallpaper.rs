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

pub fn current_wallpaper_image_blurred() -> slint::Image {
    current_wallpaper_path()
        .and_then(|path| blurred_image(&path))
        .unwrap_or_default()
}

fn boost_saturation(img: &mut image::RgbaImage, factor: f32) {
    for pixel in img.pixels_mut() {
        let [r, g, b, a] = pixel.0;
        let luma = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
        let boost = |c: u8| ((luma + (c as f32 - luma) * factor).clamp(0.0, 255.0)) as u8;
        pixel.0 = [boost(r), boost(g), boost(b), a];
    }
}

fn darken_image(img: &mut image::RgbaImage, factor: f32) {
    for pixel in img.pixels_mut() {
        pixel.0[0] = ((pixel.0[0] as f32 * factor).clamp(0.0, 255.0)) as u8;
        pixel.0[1] = ((pixel.0[1] as f32 * factor).clamp(0.0, 255.0)) as u8;
        pixel.0[2] = ((pixel.0[2] as f32 * factor).clamp(0.0, 255.0)) as u8;
    }
}

fn blurred_image(path: &Path) -> Option<slint::Image> {
    let img = image::open(path).ok()?;
    let small = img.resize_to_fill(420, 200, image::imageops::FilterType::Triangle);
    let mut blurred = small.blur(3.0).into_rgba8();
    boost_saturation(&mut blurred, 1.1);
    darken_image(&mut blurred, 0.84);
    let buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(
        blurred.as_raw(),
        blurred.width(),
        blurred.height(),
    );
    Some(slint::Image::from_rgba8(buffer))
}

fn current_wallpaper_path() -> Option<PathBuf> {
    if let Some(wallpaper) = env::var_os("SIERRA_LAUNCHER_WALLPAPER") {
        let path = PathBuf::from(wallpaper);
        if path.exists() && is_supported_image(&path) {
            return Some(path);
        }
    }

    pywal_wallpaper()
        .or_else(wallpapers_dir_wallpaper)
        .or_else(gnome_wallpaper)
        .or_else(kde_wallpaper)
        .or_else(xfce_wallpaper)
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
        .map(|ext| {
            matches!(
                ext.to_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "webp" | "bmp" | "gif" | "svg"
            )
        })
        .unwrap_or(false)
}

fn parse_file_uri(value: &str) -> Option<PathBuf> {
    let stripped = value.trim().trim_matches(|c| c == '"' || c == '\'');
    let stripped = stripped.strip_prefix("file://").unwrap_or(stripped);
    let path = PathBuf::from(stripped);
    if path.exists() { Some(path) } else { None }
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
