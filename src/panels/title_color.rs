use iced::Color;
use crate::utils::theme::Theme;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationMode {
    /// Each character cycles through all colors independently
    Rainbow,
    /// Single color wave moves from top to bottom
    Wave,
    /// All characters pulse the same color in sync
    InOutWave,
    Pulse,
    /// Random sparkle effect
    Sparkle,
    /// Gradient that shifts through colors
    Gradient,
}

pub struct TitleAnimator {
    animation_offset: usize,
    last_animation_update: Instant,
    animation_speed: Duration,
    mode: AnimationMode,
    sparkle_state: Vec<usize>, // For sparkle mode
}

impl TitleAnimator {
    pub fn new() -> Self {
        Self {
            animation_offset: 0,
            last_animation_update: Instant::now(),
            animation_speed: Duration::from_millis(100),
            mode: AnimationMode::Rainbow,
            sparkle_state: vec![0; 20], // Enough for most titles
        }
    }

    pub fn with_speed(mut self, speed_ms: u64) -> Self {
        self.animation_speed = Duration::from_millis(speed_ms);
        self
    }

    pub fn with_mode(mut self, mode: AnimationMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn set_mode(&mut self, mode: AnimationMode) {
        self.mode = mode;
        self.reset();
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_animation_update) > self.animation_speed {
            self.last_animation_update = now;
            self.animation_offset = self.animation_offset.wrapping_add(1);
            
            // Update sparkle state
            if self.mode == AnimationMode::Sparkle {
                for state in &mut self.sparkle_state {
                    if rand::random::<f32>() > 0.7 {
                        *state = state.wrapping_add(1);
                    }
                }
            }
        }
    }

    pub fn get_color_for_char(&self, theme: &Theme, char_index: usize, total_chars: usize) -> Color {
        let colors = [
            theme.color1, theme.color2, theme.color3, theme.color4,
            theme.color5, theme.color6, theme.color9, theme.color10,
            theme.color11, theme.color12, theme.color13, theme.color14,
        ];

        match self.mode {
            AnimationMode::Rainbow => {
                // Each character cycles through colors independently
                let color_index = (char_index + self.animation_offset) % colors.len();
                colors[color_index]
            }
            AnimationMode::Wave => {
                if total_chars == 0 { return theme.foreground; }
                // Single color for the whole wave, changes each cycle.
                let wave_cycle = self.animation_offset / total_chars;
                let color_index = wave_cycle % colors.len();
                let current_color = colors[color_index];

                let wave_position = self.animation_offset % total_chars;
                let distance = if char_index >= wave_position {
                    char_index - wave_position
                } else {
                    (total_chars - wave_position) + char_index
                };
                
                // Characters near the wave position get the current color
                // Others get a dimmed version of the base color
                if distance == 0 {
                    current_color
                } else if distance <= 0 {
                    Color::from_rgba(
                        current_color.r * 0.5,
                        current_color.g * 0.5,
                        current_color.b * 0.5,
                        1.0
                    )
                } else {
                    theme.foreground
                }
            }

            AnimationMode::InOutWave => {
                if total_chars == 0 { return theme.foreground; }
                // Single color for the whole wave, changes each cycle.
                let wave_cycle = self.animation_offset / total_chars;
                let color_index = wave_cycle % colors.len();
                let current_color = colors[color_index];

                let wave_position = self.animation_offset % total_chars;
                let distance = if char_index >= wave_position {
                    char_index - wave_position
                } else {
                    (total_chars - wave_position) + char_index
                };
                
                // Characters near the wave position get the current color
                // Others get a dimmed version of the base color
                if distance == 0 {
                    current_color
                } else if distance <= 14 {
                    Color::from_rgba(
                        current_color.r * 0.5,
                        current_color.g * 0.5,
                        current_color.b * 0.5,
                        1.0
                    )
                } else {
                    theme.foreground
                }
            }
            
            AnimationMode::Pulse => {
                // All characters pulse the same color in sync
                let color_index = self.animation_offset % colors.len();
                colors[color_index]
            }
            
            AnimationMode::Sparkle => {
                // Random sparkle effect
                if char_index < self.sparkle_state.len() {
                    let color_index = self.sparkle_state[char_index] % colors.len();
                    colors[color_index]
                } else {
                    theme.foreground
                }
            }
            
            AnimationMode::Gradient => {
                // Smooth gradient that shifts through colors
                let position = (char_index as f32 / total_chars as f32) * colors.len() as f32;
                let offset_position = (position + self.animation_offset as f32) % colors.len() as f32;
                let color_index = offset_position.floor() as usize % colors.len();
                let next_color_index = (color_index + 1) % colors.len();
                let blend = offset_position.fract();
                
                // Blend between two colors for smooth gradient
                let color1 = colors[color_index];
                let color2 = colors[next_color_index];
                Color::from_rgb(
                    color1.r * (1.0 - blend) + color2.r * blend,
                    color1.g * (1.0 - blend) + color2.g * blend,
                    color1.b * (1.0 - blend) + color2.b * blend,
                )
            }
        }
    }

    pub fn reset(&mut self) {
        self.animation_offset = 0;
        self.last_animation_update = Instant::now();
        self.sparkle_state = vec![0; 20];
    }

    pub fn current_mode(&self) -> AnimationMode {
        self.mode
    }
}

impl Default for TitleAnimator {
    fn default() -> Self {
        Self::new()
    }
}

// Simple random number generator for sparkle effect
mod rand {
    use std::cell::Cell;
    
    thread_local! {
        static SEED: Cell<u64> = Cell::new(0x1234_5678_9abc_def0);
    }
    
    pub fn random<T>() -> T 
    where
        T: From<f32>,
    {
        SEED.with(|seed| {
            let mut s = seed.get();
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            seed.set(s);
            T::from(s as f32 / u64::MAX as f32)
        })
    }
}