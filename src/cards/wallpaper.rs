use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

const CAROUSEL_MAX_W: u32 = 900;
const CAROUSEL_MAX_H: u32 = 500;

struct BlurCache {
    map: HashMap<PathBuf, (Vec<u8>, u32, u32)>,
}

impl BlurCache {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    fn get(&self, path: &Path) -> Option<slint::Image> {
        self.map.get(path).map(|(data, w, h)| {
            let buffer =
                slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(data, *w, *h);
            slint::Image::from_rgba8(buffer)
        })
    }

    fn spawn_preloader(paths: Vec<PathBuf>, cache: Arc<Mutex<BlurCache>>) {
        std::thread::spawn(move || {
            let mut local = BlurCache::new();
            for path in paths.iter() {
                if let Some((data, w, h)) = process_blur_raw(path) {
                    local.map.insert(path.clone(), (data, w, h));
                }
            }
            if let Ok(mut cache) = cache.lock() {
                cache.map.extend(local.map);
            }
        });
    }
}

struct ImageCache {
    map: HashMap<PathBuf, (Vec<u8>, u32, u32)>,
}

impl ImageCache {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    fn get(&self, path: &Path) -> Option<slint::Image> {
        self.map.get(path).map(|(data, w, h)| {
            let buffer =
                slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(data, *w, *h);
            slint::Image::from_rgba8(buffer)
        })
    }
}

pub struct WallpaperManager {
    paths: Vec<PathBuf>,
    index: usize,
    blur_cache: Arc<Mutex<BlurCache>>,
    image_cache: Arc<Mutex<ImageCache>>,
    pending: Arc<Mutex<HashSet<PathBuf>>>,
}

impl WallpaperManager {
    pub fn load() -> Self {
        let mut paths = wallpapers_dir_all_images().unwrap_or_default();

        let current = current_wallpaper_path();
        if let Some(cur) = &current {
            if !paths.iter().any(|p| p == cur) {
                paths.insert(0, cur.clone());
            }
        }

        let index = current
            .and_then(|cur| paths.iter().position(|p| p == &cur))
            .unwrap_or(0);

        let blur_cache = Arc::new(Mutex::new(BlurCache::new()));
        BlurCache::spawn_preloader(paths.clone(), blur_cache.clone());

        Self {
            paths,
            index,
            blur_cache,
            image_cache: Arc::new(Mutex::new(ImageCache::new())),
            pending: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn ensure_window_loaded(
        &self,
        radius: isize,
        on_loaded: impl Fn() + Send + Sync + Clone + 'static,
    ) {
        let start = self.index as isize - radius;
        let end = self.index as isize + radius;
        for off in start..=end {
            if off < 0 || off >= self.paths.len() as isize {
                continue;
            }
            let path = self.paths[off as usize].clone();
            self.spawn_load_if_needed(path, on_loaded.clone());
        }
    }

    fn spawn_load_if_needed(&self, path: PathBuf, on_loaded: impl Fn() + Send + Sync + 'static) {
        if self.image_cache.lock().unwrap().map.contains_key(&path) {
            return;
        }
        {
            let mut pending = self.pending.lock().unwrap();
            if pending.contains(&path) {
                return;
            }
            pending.insert(path.clone());
        }

        let image_cache = self.image_cache.clone();
        let pending = self.pending.clone();
        std::thread::spawn(move || {
            if let Some(raw) = process_full_raw(&path) {
                if let Ok(mut cache) = image_cache.lock() {
                    cache.map.insert(path.clone(), raw);
                }
            }
            pending.lock().unwrap().remove(&path);
            on_loaded();
        });
    }

    pub fn spawn_full_preload(&self, on_loaded: impl Fn() + Send + Sync + 'static) {
        let n = self.paths.len() as isize;
        let start = self.index as isize;
        let mut seen = HashSet::new();
        let mut order: Vec<PathBuf> = Vec::new();
        for radius in 0..n {
            for cand in [start - radius, start + radius] {
                if cand >= 0 && cand < n && seen.insert(cand) {
                    order.push(self.paths[cand as usize].clone());
                }
            }
        }

        let image_cache = self.image_cache.clone();
        let pending = self.pending.clone();
        std::thread::spawn(move || {
            for path in order {
                {
                    let mut pend = pending.lock().unwrap();
                    if image_cache.lock().unwrap().map.contains_key(&path) || pend.contains(&path) {
                        continue;
                    }
                    pend.insert(path.clone());
                }
                if let Some(raw) = process_full_raw(&path) {
                    if let Ok(mut cache) = image_cache.lock() {
                        cache.map.insert(path.clone(), raw);
                    }
                }
                pending.lock().unwrap().remove(&path);
                on_loaded();
            }
        });
    }

    fn offset_index(&self, offset: isize) -> Option<usize> {
        let idx = self.index as isize + offset;
        if idx < 0 || idx >= self.paths.len() as isize {
            None
        } else {
            Some(idx as usize)
        }
    }

    pub fn current_path(&self) -> Option<&PathBuf> {
        self.paths.get(self.index)
    }
    pub fn prev_path(&self) -> Option<&PathBuf> {
        self.offset_index(-1).and_then(|i| self.paths.get(i))
    }
    pub fn next_path(&self) -> Option<&PathBuf> {
        self.offset_index(1).and_then(|i| self.paths.get(i))
    }
    pub fn prev_prev_path(&self) -> Option<&PathBuf> {
        self.offset_index(-2).and_then(|i| self.paths.get(i))
    }
    pub fn next_next_path(&self) -> Option<&PathBuf> {
        self.offset_index(2).and_then(|i| self.paths.get(i))
    }

    pub fn select_prev(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        }
    }
    pub fn select_next(&mut self) {
        if self.index + 1 < self.paths.len() {
            self.index += 1;
        }
    }

    pub fn can_select_prev(&self) -> bool {
        self.index > 0
    }
    pub fn can_select_next(&self) -> bool {
        self.index + 1 < self.paths.len()
    }

    pub fn set_current_as_wallpaper(&self) {
        if let Some(path) = self.current_path() {
            set_wallpaper(path);
        }
    }

    fn cached_or_placeholder(&self, path: Option<&PathBuf>) -> slint::Image {
        if let Some(p) = path {
            if let Ok(cache) = self.image_cache.lock() {
                if let Some(img) = cache.get(p) {
                    return img;
                }
            }
            if let Ok(cache) = self.blur_cache.lock() {
                if let Some(img) = cache.get(p) {
                    return img;
                }
            }
        }
        slint::Image::default()
    }

    pub fn current_image(&self) -> slint::Image {
        self.cached_or_placeholder(self.current_path())
    }
    pub fn prev_image(&self) -> slint::Image {
        self.cached_or_placeholder(self.prev_path())
    }
    pub fn next_image(&self) -> slint::Image {
        self.cached_or_placeholder(self.next_path())
    }
    pub fn prev_prev_image(&self) -> slint::Image {
        self.cached_or_placeholder(self.prev_prev_path())
    }
    pub fn next_next_image(&self) -> slint::Image {
        self.cached_or_placeholder(self.next_next_path())
    }

    pub fn current_image_blurred(&self) -> slint::Image {
        if let Some(path) = self.current_path() {
            if let Ok(cache) = self.blur_cache.lock() {
                if let Some(img) = cache.get(path) {
                    return img;
                }
            }
            if let Some((data, w, h)) = process_blur_raw(path) {
                let buffer =
                    slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(&data, w, h);
                return slint::Image::from_rgba8(buffer);
            }
        }
        slint::Image::default()
    }
}

fn process_blur_raw(path: &Path) -> Option<(Vec<u8>, u32, u32)> {
    let img = image::open(path).ok()?;
    let small = img.resize_to_fill(200, 110, image::imageops::FilterType::Triangle);
    let mut blurred = small.blur(4.0).into_rgba8();
    boost_saturation(&mut blurred, 1.1);
    darken_image(&mut blurred, 0.84);
    let w = blurred.width();
    let h = blurred.height();
    Some((blurred.into_raw(), w, h))
}

fn process_full_raw(path: &Path) -> Option<(Vec<u8>, u32, u32)> {
    let img = image::open(path).ok()?;
    let img = if img.width() > CAROUSEL_MAX_W || img.height() > CAROUSEL_MAX_H {
        img.resize(
            CAROUSEL_MAX_W,
            CAROUSEL_MAX_H,
            image::imageops::FilterType::Triangle,
        )
    } else {
        img
    };
    let rgba = img.into_rgba8();
    let (w, h) = (rgba.width(), rgba.height());
    Some((rgba.into_raw(), w, h))
}

fn blurred_image(path: &Path) -> Option<slint::Image> {
    process_blur_raw(path).map(|(data, w, h)| {
        let buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(&data, w, h);
        slint::Image::from_rgba8(buffer)
    })
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

fn wallpapers_dir_all_images() -> Option<Vec<PathBuf>> {
    let dir = wallpaper_directory()?;
    let mut images: Vec<PathBuf> = fs::read_dir(&dir)
        .ok()?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.is_file() && is_supported_image(path))
        .collect();
    images.sort();
    Some(images)
}

fn wallpaper_directory() -> Option<PathBuf> {
    config_wallpaper_dir().or_else(default_wallpaper_dir)
}

fn config_wallpaper_dir() -> Option<PathBuf> {
    let contents = read_config_file()?;
    for line in contents.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        if let Some(eq) = t.find('=') {
            let key = t[..eq].trim();
            if key != "wallpaper_dir" {
                continue;
            }
            let value = t[eq + 1..].trim();
            let path = expand_path(value)?;
            if path.exists() && path.is_dir() {
                return Some(path);
            }
        }
    }
    None
}

fn default_wallpaper_dir() -> Option<PathBuf> {
    let home = env::var_os("HOME")?;
    let dir = PathBuf::from(home).join("Pictures/Wallpapers");
    if dir.exists() && dir.is_dir() {
        Some(dir)
    } else {
        None
    }
}

fn current_wallpaper_path() -> Option<PathBuf> {
    if let Some(wallpaper) = env::var_os("SIERRA_LAUNCHER_WALLPAPER") {
        let path = PathBuf::from(wallpaper);
        if path.exists() && is_supported_image(&path) {
            return Some(path);
        }
    }
    if let Some(p) = pywal_wallpaper() {
        return Some(p);
    }
    if let Some(p) = config_wallpaper_path() {
        return Some(p);
    }
    if let Some(home) = env::var_os("HOME") {
        let candidate_dir = PathBuf::from(home).join("Wallpaper");
        if candidate_dir.exists() && candidate_dir.is_dir() {
            if let Some(p) = first_image_in_dir(&candidate_dir) {
                return Some(p);
            }
        }
    }
    wallpapers_dir_wallpaper()
}

fn config_wallpaper_path() -> Option<PathBuf> {
    let contents = read_config_file()?;
    for line in contents.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        let path_value = if let Some(eq) = t.find('=') {
            let key = t[..eq].trim();
            if key != "wallpaper_dir" {
                continue;
            }
            t[eq + 1..].trim()
        } else {
            t
        };
        let path = expand_path(path_value)?;
        if path.exists() {
            if path.is_file() && is_supported_image(&path) {
                return Some(path);
            }
            if path.is_dir() {
                return first_image_in_dir(&path);
            }
        }
    }
    None
}

fn config_cache_dir() -> Option<PathBuf> {
    let contents = read_config_file()?;
    for line in contents.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        if let Some(eq) = t.find('=') {
            let key = t[..eq].trim();
            if key != "cache_dir" {
                continue;
            }
            let value = t[eq + 1..].trim();
            return expand_path(value);
        }
    }
    None
}

fn read_config_file() -> Option<String> {
    let home = env::var_os("HOME")?;
    let cfg = PathBuf::from(home).join(".config/sierra_launcher/sierra");
    fs::read_to_string(cfg).ok()
}

fn expand_path(value: &str) -> Option<PathBuf> {
    let mut value = value.trim();
    if (value.starts_with('"') && value.ends_with('"'))
        || (value.starts_with('\'') && value.ends_with('\''))
    {
        value = &value[1..value.len() - 1];
    }
    if value.starts_with("~/") {
        let home = env::var_os("HOME")?;
        return Some(PathBuf::from(home).join(&value[2..]));
    }
    if value == "~" {
        return env::var_os("HOME").map(PathBuf::from);
    }
    Some(PathBuf::from(value))
}

fn first_image_in_dir(dir: &Path) -> Option<PathBuf> {
    let mut images: Vec<PathBuf> = fs::read_dir(dir)
        .ok()?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.is_file() && is_supported_image(path))
        .collect();
    images.sort();
    images.into_iter().next()
}

fn pywal_wallpaper() -> Option<PathBuf> {
    let cache_dir = config_cache_dir().unwrap_or_else(default_cache_dir);
    if let Some(path) = pywal_colors_json(&cache_dir) {
        return Some(path);
    }
    if let Some(path) = pywal_wal_file(&cache_dir) {
        return Some(path);
    }
    None
}

fn default_cache_dir() -> PathBuf {
    let home = env::var_os("HOME").unwrap_or_default();
    PathBuf::from(home).join(".cache/wal")
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
    let wallpapers_dir = wallpaper_directory()?;
    let entries = fs::read_dir(&wallpapers_dir).ok()?;
    let mut images: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.is_file() && is_supported_image(path))
        .collect();
    images.sort_by_key(|path| fs::metadata(path).and_then(|m| m.modified()).ok());
    images.pop()
}

fn wallpapers_dir_match(path: &Path) -> Option<PathBuf> {
    let wallpapers_dir = wallpaper_directory()?;
    let basename = path.file_name()?;
    fs::read_dir(&wallpapers_dir)
        .ok()?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .find(|candidate| candidate.file_name() == Some(basename) && is_supported_image(candidate))
}

pub fn set_wallpaper(path: &Path) {
    let path_str = path.to_string_lossy().into_owned();
    unsafe { env::set_var("SIERRA_LAUNCHER_WALLPAPER", &path_str) };
    let _ = std::process::Command::new("wal")
        .arg("-i")
        .arg(path)
        .arg("-n")
        .status();
    let _ = std::process::Command::new("pkill")
        .arg("-x")
        .arg("swaybg")
        .status();
    let _ = std::process::Command::new("swaybg")
        .arg("-m")
        .arg("fill")
        .arg("-i")
        .arg(path)
        .spawn();
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
