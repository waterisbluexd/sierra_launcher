use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct WallpaperManager {
    wallpaper_dir: PathBuf,
    cache_dir: PathBuf,
}

/* ============================
   Index data structures
   ============================ */

#[derive(Debug, Serialize, Deserialize)]
pub struct WallpaperIndex {
    pub wallpaper_dir: PathBuf,
    pub generated_at: u64,
    pub wallpapers: Vec<WallpaperEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WallpaperEntry {
    pub name: String,
    pub path: PathBuf,
    pub kind: WallpaperKind,
    pub thumbnail: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WallpaperKind {
    Image,
    Video,
}

/* ============================
   Implementation
   ============================ */

impl WallpaperManager {
    pub fn new(wallpaper_dir: PathBuf) -> Self {
        let cache_dir = Self::default_cache_dir();
        Self {
            wallpaper_dir,
            cache_dir,
        }
    }

    /// Main entry point:
    /// - creates cache dirs
    /// - generates thumbnails for videos (once)
    /// - writes index.json
    pub fn ensure_cache(&self) {
        if fs::create_dir_all(&self.cache_dir).is_err() {
            return;
        }

        let thumbs_dir = self.cache_dir.join("thumbs");
        let _ = fs::create_dir_all(&thumbs_dir);

        let mut wallpapers = Vec::new();

        let entries = match fs::read_dir(&self.wallpaper_dir) {
            Ok(e) => e,
            Err(_) => return, // silent fail by design
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let name = match path.file_name() {
                Some(n) => n.to_string_lossy().to_string(),
                None => continue,
            };

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            match ext.as_str() {
                "mp4" | "mkv" | "webm" | "avi" => {
                    let thumb_path = thumbs_dir.join(format!("{}.png", name));

                    if !thumb_path.exists() {
                        Self::generate_thumbnail(&path, &thumb_path);
                    }

                    wallpapers.push(WallpaperEntry {
                        name,
                        path,
                        kind: WallpaperKind::Video,
                        thumbnail: Some(thumb_path),
                    });
                }
                _ => {
                    wallpapers.push(WallpaperEntry {
                        name,
                        path,
                        kind: WallpaperKind::Image,
                        thumbnail: None,
                    });
                }
            }
        }

        let generated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let index = WallpaperIndex {
            wallpaper_dir: self.wallpaper_dir.clone(),
            generated_at,
            wallpapers,
        };

        let index_path = self.cache_dir.join("index.json");

        if let Ok(json) = serde_json::to_string_pretty(&index) {
            let _ = fs::write(index_path, json);
        }
    }

    /// Load index.json from cache
    pub fn load_index(&self) -> Option<WallpaperIndex> {
        let index_path = self.cache_dir.join("index.json");
        let content = fs::read_to_string(index_path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Extract first frame of video using ffmpeg
    fn generate_thumbnail(video: &PathBuf, thumbnail: &PathBuf) {
        let _ = Command::new("ffmpeg")
            .args([
                "-y",
                "-loglevel",
                "error",
                "-i",
                video.to_str().unwrap(),
                "-frames:v",
                "1",
                thumbnail.to_str().unwrap(),
            ])
            .status();
    }

    /// ~/.cache/sierra/wallpapers
    fn default_cache_dir() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join(".cache")
                .join("sierra")
                .join("wallpapers")
        } else {
            PathBuf::from(".cache/sierra/wallpapers")
        }
    }
}
