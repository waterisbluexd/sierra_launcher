use mpris::{Player, PlayerFinder, PlaybackStatus};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct MusicPlayerState {
    pub app_name: String,
    pub song_name: String,
    pub artist_name: String,
    pub current_time: f32,      // in seconds
    pub total_time: f32,        // in seconds
    pub is_playing: bool,
    pub player_available: bool,
    // Internal state (not cloned to messages)
    player: Option<Player>,
    last_check: std::time::Instant,
    cached_position: Option<i64>,
    last_status: Option<PlaybackStatus>,
}

impl Default for MusicPlayerState {
    fn default() -> Self {
        Self {
            app_name: "No Player".to_string(),
            song_name: "No Music Playing".to_string(),
            artist_name: "Start playing music in Spotify, YouTube, or any MPRIS-compatible player".to_string(),
            current_time: 0.0,
            total_time: 0.0,
            is_playing: false,
            player_available: false,
            player: None,
            last_check: std::time::Instant::now(),
            cached_position: None,
            last_status: None,
        }
    }
}

impl MusicPlayerState {
    pub fn new() -> Self {
        let mut state = Self::default();
        state.refresh_player();
        state
    }

    fn find_active_player() -> Option<Player> {
        let finder = PlayerFinder::new().ok()?;

        // Try to find an active player (playing or paused)
        for player in finder.find_all().ok()? {
            if let Ok(status) = player.get_playback_status() {
                if status == PlaybackStatus::Playing || status == PlaybackStatus::Paused {
                    return Some(player);
                }
            }
        }

        None
    }

    pub fn refresh_player(&mut self) {
        // Refresh player every 2 seconds
        if self.last_check.elapsed() > Duration::from_secs(2) {
            self.player = Self::find_active_player();
            self.last_check = std::time::Instant::now();
            
            // Update player info
            self.update_player_info();
        }
    }

    pub fn update_player_info(&mut self) {
        if let Some(player) = &self.player {
            if let Ok(metadata) = player.get_metadata() {
                if let Ok(status) = player.get_playback_status() {
                    // Update app name
                    self.app_name = player.identity().to_string();

                    // Update song name
                    self.song_name = metadata.title()
                        .unwrap_or("Unknown")
                        .to_string();

                    // Update artist
                    self.artist_name = metadata
                        .artists()
                        .and_then(|artists| artists.first().map(|s| s.to_string()))
                        .unwrap_or_else(|| "Unknown Artist".to_string());

                    // Update total time
                    self.total_time = metadata.length()
                        .map(|l| (l.as_micros() as f64 / 1_000_000.0) as f32)
                        .unwrap_or(0.0);

                    // Update current position
                    let position = if status == PlaybackStatus::Paused {
                        // If we just transitioned to paused, cache the current position
                        if self.last_status != Some(PlaybackStatus::Paused) {
                            self.cached_position = player.get_position()
                                .ok()
                                .map(|p| p.as_micros() as i64);
                        }
                        // Use cached position
                        self.cached_position.unwrap_or(0)
                    } else {
                        // Playing or stopped - get fresh position and clear cache
                        self.cached_position = None;
                        player.get_position()
                            .ok()
                            .map(|p| p.as_micros() as i64)
                            .unwrap_or(0)
                    };

                    self.current_time = (position as f64 / 1_000_000.0) as f32;

                    // Update playing status
                    self.is_playing = status == PlaybackStatus::Playing;
                    self.last_status = Some(status);
                    self.player_available = true;
                    return;
                }
            }
        }

        // No player available
        self.player_available = false;
        self.app_name = "No Player".to_string();
        self.song_name = "No Music Playing".to_string();
        self.artist_name = "Start playing music in Spotify, YouTube, or any MPRIS-compatible player".to_string();
        self.current_time = 0.0;
        self.total_time = 0.0;
        self.is_playing = false;
    }

    pub fn format_time(seconds: f32) -> String {
        let total_seconds = seconds as u32;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let secs = total_seconds % 60;

        if hours > 0 {
            format!("{}:{:02}:{:02}", hours, minutes, secs)
        } else {
            format!("{}:{:02}", minutes, secs)
        }
    }

    pub fn play_pause(&mut self) -> bool {
        if let Some(player) = &self.player {
            if player.play_pause().is_ok() {
                self.cached_position = None;
                std::thread::sleep(Duration::from_millis(100));
                self.update_player_info();
                return true;
            }
        }
        false
    }

    pub fn next_track(&mut self) -> bool {
        if let Some(player) = &self.player {
            let result = if let Err(_) = player.next() {
                // If next() fails, try seeking forward significantly
                let offset = Duration::from_secs(999999);
                player.seek_forwards(&offset).is_ok()
            } else {
                true
            };

            if result {
                self.cached_position = None;
                std::thread::sleep(Duration::from_millis(100));
                self.refresh_player();
                return true;
            }
        }
        false
    }

    pub fn previous_track(&mut self) -> bool {
        if let Some(player) = &self.player {
            let result = if let Err(_) = player.previous() {
                // If previous() fails, try seeking backwards significantly
                let offset = Duration::from_secs(999999);
                player.seek_backwards(&offset).is_ok()
            } else {
                true
            };

            if result {
                self.cached_position = None;
                std::thread::sleep(Duration::from_millis(100));
                self.refresh_player();
                return true;
            }
        }
        false
    }

    pub fn seek_to(&mut self, position_seconds: f32) -> bool {
        if let Some(player) = &self.player {
            let position_micros = (position_seconds * 1_000_000.0) as i64;
            
            // MPRIS uses absolute position
            if let Ok(metadata) = player.get_metadata() {
                if let Some(track_id) = metadata.track_id() {
                    // Try to set position
                    let position = Duration::from_micros(position_micros as u64);
                    if player.set_position(track_id, &position).is_ok() {
                        self.current_time = position_seconds;
                        self.cached_position = Some(position_micros);
                        return true;
                    }
                }
            }
        }
        false
    }
}