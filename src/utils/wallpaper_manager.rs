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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperIndex {
    pub wallpaper_dir: PathBuf,
    pub generated_at: u64,
    pub wallpapers: Vec<WallpaperEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperEntry {
    pub name: String,
    pub path: PathBuf,
    pub kind: WallpaperKind,
    /// ✅ NOW ALL ENTRIES HAVE THUMBNAILS (images + videos)
    pub thumbnail: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// - generates thumbnails for ALL wallpapers (images + videos)
    /// - writes index.json
    pub fn ensure_cache(&self) {
        if fs::create_dir_all(&self.cache_dir).is_err() {
            eprintln!("[Wallpaper] Failed to create cache dir");
            return;
        }

        let thumbs_dir = self.cache_dir.join("thumbs");
        let _ = fs::create_dir_all(&thumbs_dir);

        let mut wallpapers = Vec::new();

        let entries = match fs::read_dir(&self.wallpaper_dir) {
            Ok(e) => e,
            Err(_) => {
                eprintln!("[Wallpaper] Cannot read wallpaper dir");
                return;
            }
        };

        eprintln!("[Wallpaper] Scanning wallpapers...");

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

            let (kind, needs_ffmpeg) = match ext.as_str() {
                "mp4" | "mkv" | "webm" | "avi" => (WallpaperKind::Video, true),
                "jpg" | "jpeg" | "png" | "webp" | "bmp" => (WallpaperKind::Image, false),
                _ => continue, // skip unknown formats
            };

            // ✅ GENERATE THUMBNAIL FOR EVERYTHING
            let thumb_path = thumbs_dir.join(format!("{}.jpg", name));

            if !thumb_path.exists() {
                if needs_ffmpeg {
                    Self::generate_video_thumbnail(&path, &thumb_path);
                } else {
                    Self::generate_image_thumbnail(&path, &thumb_path);
                }
            }

            wallpapers.push(WallpaperEntry {
                name,
                path,
                kind,
                thumbnail: thumb_path,
            });
        }

        eprintln!("[Wallpaper] Processed {} wallpapers", wallpapers.len());

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
            let _ = fs::write(&index_path, json);
            eprintln!("[Wallpaper] Index saved to {:?}", index_path);
        }
    }

    /// Load index.json from cache
    pub fn load_index(&self) -> Option<WallpaperIndex> {
        let index_path = self.cache_dir.join("index.json");
        let content = fs::read_to_string(&index_path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// ✅ NEW: Generate thumbnail from image using `image` crate (FAST)
    fn generate_image_thumbnail(source: &PathBuf, thumbnail: &PathBuf) {
        use image::ImageReader;
        
        // Try to open and decode image
        let img = match ImageReader::open(source) {
            Ok(reader) => match reader.decode() {
                Ok(img) => img,
                Err(e) => {
                    eprintln!("[Wallpaper] Failed to decode {:?}: {}", source, e);
                    return;
                }
            },
            Err(e) => {
                eprintln!("[Wallpaper] Failed to open {:?}: {}", source, e);
                return;
            }
        };

        // Resize to 480x270 (16:9 thumbnail - fast preview)
        let thumb = img.resize(480, 270, image::imageops::FilterType::Lanczos3);

        // Save as JPEG with 85% quality
        if let Err(e) = thumb.save_with_format(thumbnail, image::ImageFormat::Jpeg) {
            eprintln!("[Wallpaper] Failed to save thumbnail {:?}: {}", thumbnail, e);
        } else {
            eprintln!("[Wallpaper] ✓ Generated thumbnail: {:?}", thumbnail);
        }
    }

    /// Extract first frame of video using ffmpeg
    fn generate_video_thumbnail(video: &PathBuf, thumbnail: &PathBuf) {
        eprintln!("[Wallpaper] Generating video thumbnail: {:?}", video);
        
        let status = Command::new("ffmpeg")
            .args([
                "-y",
                "-loglevel",
                "error",
                "-i",
                video.to_str().unwrap(),
                "-vf",
                "scale=480:270", // ✅ Match image thumbnail size
                "-frames:v",
                "1",
                "-q:v",
                "5", // JPEG quality
                thumbnail.to_str().unwrap(),
            ])
            .status();

        match status {
            Ok(s) if s.success() => eprintln!("[Wallpaper] ✓ Generated video thumbnail: {:?}", thumbnail),
            _ => eprintln!("[Wallpaper] Failed to generate video thumbnail for {:?}", video),
        }
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