use iced::widget::{container, text, column, row};
use iced::{Element, Border, Color, Length, Alignment};
use crate::utils::theme::Theme;
use crate::Message;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::fs;

/// System panel with real-time metrics
pub struct SystemPanel {
    metrics: Arc<Mutex<SystemMetrics>>,
}

/// Represents system metrics data
#[derive(Clone, Debug)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub mem_usage: f32,
    pub net_usage: f32,
    pub disk_usage: f32,
    pub gpu_usage: f32,
    pub gpu1_usage: f32,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            mem_usage: 0.0,
            net_usage: 0.0,
            disk_usage: 0.0,
            gpu_usage: 0.0,
            gpu1_usage: 0.0,
        }
    }
}

impl SystemPanel {
    pub fn new() -> Self {
        let metrics = Arc::new(Mutex::new(SystemMetrics::default()));
        let metrics_clone = Arc::clone(&metrics);

        // Spawn background thread to collect system metrics
        thread::spawn(move || {
            loop {
                let mut m = metrics_clone.lock().unwrap();

                // Simple CPU usage from /proc/stat
                m.cpu_usage = Self::get_cpu_usage();

                // Memory usage from /proc/meminfo
                m.mem_usage = Self::get_memory_usage();

                // Disk usage from df
                m.disk_usage = Self::get_disk_usage();

                // Network usage (simplified)
                m.net_usage = Self::get_network_usage();

                // GPU usage (nvidia-smi)
                let gpu_usages = Self::get_gpu_usage();
                m.gpu_usage = gpu_usages[0];
                m.gpu1_usage = gpu_usages[1];

                drop(m);
                thread::sleep(Duration::from_secs(2));
            }
        });

        Self { metrics }
    }

    fn get_cpu_usage() -> f32 {
        // Read from /proc/loadavg for a simple metric
        if let Ok(loadavg) = fs::read_to_string("/proc/loadavg") {
            if let Some(first) = loadavg.split_whitespace().next() {
                if let Ok(load) = first.parse::<f32>() {
                    // Convert load average to percentage (assuming 4 cores)
                    return (load * 25.0).min(100.0);
                }
            }
        }
        0.0
    }

    fn get_memory_usage() -> f32 {
        // Read from /proc/meminfo
        if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
            let mut total = 0u64;
            let mut available = 0u64;
            
            for line in meminfo.lines() {
                if line.starts_with("MemTotal:") {
                    total = line.split_whitespace().nth(1)
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                } else if line.starts_with("MemAvailable:") {
                    available = line.split_whitespace().nth(1)
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                }
            }
            
            if total > 0 {
                return ((total - available) as f64 / total as f64 * 100.0) as f32;
            }
        }
        0.0
    }

    fn get_disk_usage() -> f32 {
        // Use df command
        if let Ok(output) = std::process::Command::new("df")
            .args(&["-h", "/"])
            .output() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                for line in stdout.lines().skip(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 5 {
                        if let Some(percent_str) = parts[4].strip_suffix('%') {
                            if let Ok(percent) = percent_str.parse::<f32>() {
                                return percent;
                            }
                        }
                    }
                }
            }
        }
        0.0
    }

    fn get_network_usage() -> f32 {
        // Read from /proc/net/dev
        if let Ok(netdev) = fs::read_to_string("/proc/net/dev") {
            let mut total_bytes = 0u64;
            
            for line in netdev.lines().skip(2) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 10 {
                    // Received bytes + transmitted bytes
                    let rx: u64 = parts[1].parse().unwrap_or(0);
                    let tx: u64 = parts[9].parse().unwrap_or(0);
                    total_bytes += rx + tx;
                }
            }
            
            // Convert to percentage (arbitrary scaling)
            return ((total_bytes as f64 / 100_000_000.0) * 100.0).min(100.0) as f32;
        }
        0.0
    }

    fn get_gpu_usage() -> Vec<f32> {
        let mut gpu_usages = vec![0.0, 0.0];
        if let Ok(output) = std::process::Command::new("nvidia-smi")
            .args(&["--query-gpu=utilization.gpu", "--format=csv,noheader,nounits"])
            .output() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                for (i, line) in stdout.lines().take(2).enumerate() {
                    if let Ok(usage) = line.trim().parse::<f32>() {
                        gpu_usages[i] = usage;
                    }
                }
            }
        }
        gpu_usages
    }

    pub fn view<'a>(
        &'a self,
        theme: &'a Theme,
        bg_with_alpha: Color,
        font: iced::Font,
        font_size: f32,
    ) -> Element<'a, Message> {
        let metrics = self.metrics.lock().unwrap().clone();

        let metrics_data = vec![
            ("CPU", metrics.cpu_usage),
            ("MEM", metrics.mem_usage),
            ("NET", metrics.net_usage),
            ("DIS", metrics.disk_usage),
            ("GPU", metrics.gpu_usage),
            ("GPU1", metrics.gpu1_usage),
        ];

        // Create the row of vertical bars with equal spacing
        let bars_row = row(
            metrics_data
                .into_iter()
                .map(|(label, value)| vertical_bar(label, value, theme))
                .collect::<Vec<_>>()
        )
        .spacing(12)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([20, 16]);

        container(
            container(
                iced::widget::stack![
                    // Main content container with border
                    container(
                        container(bars_row)
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .padding(iced::padding::top(25))
                            .style(move |_| container::Style {
                                background: None,
                                border: Border {
                                    color: theme.color3,
                                    width: 2.0,
                                    radius: 0.0.into(),
                                },
                                ..Default::default()
                            })
                    )
                    .padding(iced::padding::top(15))
                    .width(Length::Fill)
                    .height(Length::Fill),
                    
                    // Floating title label
                    container(
                        container(
                            text(" System ")
                                .color(theme.color6)
                                .font(font)
                                .size(font_size)
                        )
                        .width(Length::Shrink)
                        .height(Length::Shrink)
                        .style(move |_| container::Style {
                            background: Some(bg_with_alpha.into()),
                            ..Default::default()
                        })
                    )
                    .padding(iced::padding::left(8).top(5))
                    .width(Length::Shrink)
                    .height(Length::Shrink),
                ]
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: None,
                ..Default::default()
            }),
        )
        .width(Length::Fill)
        .height(Length::FillPortion(1))
        .style(move |_| container::Style {
            background: None,
            ..Default::default()
        })
        .into()
    }
}

/// Creates a vertical bar visualization for a metric
fn vertical_bar<'a>(
    label: &'a str,
    value: f32,
    theme: &'a Theme,
) -> Element<'a, Message> {
    let percentage_text = text(format!("{:.0}%", value))
        .size(14)
        .color(Color::WHITE)
        .width(Length::Fill)
        .center();

    // Create the vertical bar using a container
    let bar_height_ratio = (value / 100.0).clamp(0.0, 1.0);
    
    // Bar container - we'll use two containers to simulate filled/empty portions
    let bar_visual = container(
        column![
            // Empty portion (top)
            container(
                container("")
                    .width(Length::Fixed(24.0))
                    .height(Length::FillPortion(((1.0 - bar_height_ratio) * 100.0) as u16))
            )
            .width(Length::Fixed(24.0))
            .style(move |_| container::Style {
                background: Some(Color::from_rgb(0.2, 0.2, 0.2).into()),
                border: Border {
                    color: Color::from_rgb(0.3, 0.3, 0.3),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }),
            // Filled portion (bottom)
            container(
                container("")
                    .width(Length::Fixed(24.0))
                    .height(Length::FillPortion((bar_height_ratio * 100.0) as u16))
            )
            .width(Length::Fixed(24.0))
            .style(move |_| container::Style {
                background: Some(theme.color6.into()),
                border: Border {
                    color: theme.color6,
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }),
        ]
        .spacing(0)
        .width(Length::Fixed(24.0))
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill);

    let label_text = text(label)
        .size(12)
        .color(theme.color3)
        .width(Length::Fill)
        .center();

    // Assemble the column with spacing
    column![
        percentage_text,
        container("").height(Length::Fixed(4.0)),
        bar_visual,
        container("").height(Length::Fixed(4.0)),
        label_text,
    ]
    .spacing(0)
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(Alignment::Center)
    .into()
}