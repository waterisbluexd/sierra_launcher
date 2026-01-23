use iced::widget::{container, text, column, row, stack, Space};
use iced::{Element, Border, Color, Length, Alignment, Font};
use crate::utils::theme::Theme;
use crate::Message;

use sysinfo::{System, Disks, Networks};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub struct SystemPanel {
    metrics: Arc<Mutex<Option<SystemMetrics>>>,
    started: bool,
}

#[derive(Clone)]
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
        Self {
            metrics: Arc::new(Mutex::new(None)),
            started: false,
        }
    }

    /// 🔥 Start system monitoring AFTER UI is rendered
    pub fn start(&mut self) {
        if self.started {
            return;
        }
        self.started = true;

        let metrics = Arc::clone(&self.metrics);

        thread::spawn(move || {
            eprintln!("[System] Initializing system monitoring...");
            let start = Instant::now();

            // EXPENSIVE — now fully deferred
            let mut sys = System::new_all();
            let mut networks = Networks::new_with_refreshed_list();
            let mut disks = Disks::new_with_refreshed_list();

            eprintln!("[System] ✓ Initialized in {:?}", start.elapsed());

            // Enable UI rendering
            *metrics.lock().unwrap() = Some(SystemMetrics::default());

            loop {
                sys.refresh_cpu_usage();
                sys.refresh_memory();
                networks.refresh();
                disks.refresh();

                let mut guard = metrics.lock().unwrap();
                if let Some(ref mut m) = *guard {
                    m.cpu_usage = sys.global_cpu_usage();

                    let total_mem = sys.total_memory();
                    m.mem_usage = if total_mem > 0 {
                        (sys.used_memory() as f64 / total_mem as f64 * 100.0) as f32
                    } else {
                        0.0
                    };

                    let (disk_used, disk_total) = disks.list().iter().fold(
                        (0u64, 0u64),
                        |(u, t), d| {
                            let total = d.total_space();
                            let used = total - d.available_space();
                            (u + used, t + total)
                        },
                    );

                    m.disk_usage = if disk_total > 0 {
                        (disk_used as f64 / disk_total as f64 * 100.0) as f32
                    } else {
                        0.0
                    };

                    let net_total: u64 = networks
                        .iter()
                        .map(|(_, n)| n.received() + n.transmitted())
                        .sum();

                    m.net_usage = (net_total as f64 / 10_000_000.0 * 100.0)
                        .min(100.0) as f32;

                    let gpus = Self::get_gpu_usage_cached();
                    m.gpu_usage = gpus[0];
                    m.gpu1_usage = gpus[1];
                }

                drop(guard);
                thread::sleep(Duration::from_secs(2));
            }
        });
    }

    fn get_gpu_usage_cached() -> Vec<f32> {
        use std::sync::OnceLock;

        static CACHE: OnceLock<Mutex<(Vec<f32>, Instant)>> = OnceLock::new();
        let cache = CACHE.get_or_init(|| {
            Mutex::new((vec![0.0, 0.0], Instant::now() - Duration::from_secs(10)))
        });

        let mut guard = cache.lock().unwrap();
        if guard.1.elapsed() > Duration::from_secs(4) {
            guard.0 = Self::get_gpu_usage();
            guard.1 = Instant::now();
        }

        guard.0.clone()
    }

    fn get_gpu_usage() -> Vec<f32> {
        let mut gpu_usages = vec![0.0, 0.0];

        let output = std::process::Command::new("timeout")
            .args([
                "1",
                "nvidia-smi",
                "--query-gpu=utilization.gpu",
                "--format=csv,noheader,nounits",
            ])
            .output();

        if let Ok(o) = output {
            if let Ok(text) = String::from_utf8(o.stdout) {
                for (i, line) in text.lines().take(2).enumerate() {
                    if let Ok(v) = line.trim().parse::<f32>() {
                        gpu_usages[i] = v;
                    }
                }
            }
        }

        gpu_usages
    }

    pub fn metrics(&self) -> Option<SystemMetrics> {
        self.metrics.lock().unwrap().clone()
    }
}

#[inline]
fn vertical_bar<'a>(
    label: &'a str,
    value: f32,
    theme: &'a Theme,
    font: Font,
) -> Element<'a, Message> {
    const BAR_WIDTH: f32 = 20.0;

    let percentage_text = text(format!("{:.0}%", value))
        .size(12)
        .font(font)
        .color(Color::WHITE)
        .width(Length::Fill)
        .center();

    let ratio = (value / 100.0).clamp(0.0, 1.0);
    let filled = (ratio * 1000.0).round() as u16;
    let empty = 1000u16.saturating_sub(filled);

    let bar_visual = container(
        column![
            container(Space::new())
                .width(Length::Fixed(BAR_WIDTH))
                .height(Length::FillPortion(empty))
                .style(move |_| container::Style {
                    background: Some(theme.color11.into()),
                    ..Default::default()
                }),
            container(Space::new())
                .width(Length::Fixed(BAR_WIDTH))
                .height(Length::FillPortion(filled))
                .style(move |_| container::Style {
                    background: Some(theme.color6.into()),
                    ..Default::default()
                }),
        ]
        .width(Length::Fixed(BAR_WIDTH))
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill);

    column![
        percentage_text,
        bar_visual,
        text(label)
            .size(12)
            .font(font)
            .color(theme.color3)
            .width(Length::Fill)
            .center()
    ]
    .spacing(4)
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(Alignment::Center)
    .into()
}

pub fn system_panel_view<'a>(
    system_panel: &'a SystemPanel,
    theme: &'a Theme,
    bg_with_alpha: Color,
    font: Font,
    font_size: f32,
) -> Element<'a, Message> {
    let metrics = system_panel.metrics();

    if metrics.is_none() {
        return container(
            stack![
                container(
                    text("Loading system info...")
                        .font(font)
                        .size(font_size)
                        .color(theme.color6)
                        .center()
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),

                container(
                    text(" System ")
                        .font(font)
                        .size(font_size)
                        .color(theme.color6)
                )
                .padding([5, 8])
                .style(move |_| container::Style {
                    background: Some(bg_with_alpha.into()),
                    ..Default::default()
                })
            ]
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into();
    }

    let m = metrics.unwrap();
    let data = [
        ("CPU", m.cpu_usage),
        ("MEM", m.mem_usage),
        ("NET", m.net_usage),
        ("DISK", m.disk_usage),
        ("GPU0", m.gpu_usage),
        ("GPU1", m.gpu1_usage),
    ];

    let bars = row(
        data.iter()
            .map(|&(l, v)| vertical_bar(l, v, theme, font))
            .collect::<Vec<_>>(),
    )
    .spacing(12)
    .padding(8)
    .width(Length::Fill)
    .height(Length::Fill);

    container(
        stack![
            container(bars)
                .padding(iced::padding::top(25))
                .style(move |_| container::Style {
                    border: Border {
                        color: theme.color3,
                        width: 2.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                }),

            container(
                text(" System ")
                    .font(font)
                    .size(font_size)
                    .color(theme.color6)
            )
            .padding([5, 8])
            .style(move |_| container::Style {
                background: Some(bg_with_alpha.into()),
                ..Default::default()
            })
        ]
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
