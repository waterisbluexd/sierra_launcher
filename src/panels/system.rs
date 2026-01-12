use iced::widget::{container, text, column, row, stack, Space};
use iced::{Element, Border, Color, Length, Alignment, Font};
use crate::utils::theme::Theme;
use crate::Message;
use sysinfo::{System, Disks, Networks}; 
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// System panel with real-time metrics
pub struct SystemPanel {
    metrics: Arc<Mutex<SystemMetrics>>,
}

/// Represents system metrics data
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
        let metrics = Arc::new(Mutex::new(SystemMetrics::default()));
        let metrics_clone = Arc::clone(&metrics);

        // Spawn background thread to collect system metrics
        thread::spawn(move || {
            let mut sys = System::new_all();
            let mut networks = Networks::new_with_refreshed_list();
            let mut disks = Disks::new_with_refreshed_list();

            loop {
                // Refresh system info
                sys.refresh_cpu_usage();
                sys.refresh_memory();
                networks.refresh();
                disks.refresh();

                let mut m = metrics_clone.lock().unwrap();

                // CPU usage
                m.cpu_usage = sys.global_cpu_usage();

                // Memory usage
                m.mem_usage = if sys.total_memory() > 0 {
                    (sys.used_memory() as f64 / sys.total_memory() as f64 * 100.0) as f32
                } else {
                    0.0
                };

                // Disk usage
                let (disk_used, disk_total): (u64, u64) = disks.list().iter()
                   .map(|disk| (disk.total_space() - disk.available_space(), disk.total_space()))
                   .fold((0, 0), |(acc_used, acc_total), (used, total)| {
                        (acc_used + used, acc_total + total)
                    });
                m.disk_usage = if disk_total > 0 {
                    (disk_used as f64 / disk_total as f64 * 100.0) as f32
                } else {
                    0.0
                };

                // Network usage
                let total_network: u64 = networks.iter()
                   .map(|(_, data)| data.received() + data.transmitted())
                   .sum();
                m.net_usage = (total_network as f64 / 10_000_000.0 * 100.0) as f32;

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
}

/// Creates a vertical bar visualization for a metric
fn vertical_bar<'a>(
    label: &'a str,
    value: f32,
    theme: &'a Theme,
    font: Font,
) -> Element<'a, Message> {
    let percentage_text = text(format!("{:.0}%", value))
       .size(12)
       .font(font)
       .color(Color::WHITE)
       .width(Length::Fill)
       .center();

    let bar_height_ratio = (value / 100.0).clamp(0.0, 1.0);
    
    // Calculate portions - these need to be at least 1 to be visible
    let empty_portion = ((1.0 - bar_height_ratio) * 100.0).max(1.0) as u16;
    let filled_portion = (bar_height_ratio * 100.0).max(1.0) as u16;
    
    let bar_visual = container(
        column![
            // Empty/background portion (top)
            container(
                Space::new()
                   .width(24.0)
                   .height(Length::FillPortion(empty_portion))
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
                Space::new()
                   .width(24.0)
                   .height(Length::FillPortion(filled_portion))
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
    font: iced::Font,
    font_size: f32,
) -> Element<'a, Message> {
    let metrics = system_panel.metrics.lock().unwrap().clone();

    let metrics_data = vec![
        ("CPU", metrics.cpu_usage),
        ("MEM", metrics.mem_usage),
        ("NET", metrics.net_usage),
        ("DISK", metrics.disk_usage),
        ("GPU0", metrics.gpu_usage),
        ("GPU1", metrics.gpu1_usage),
    ];

    let bars_row = row(
        metrics_data
           .into_iter()
           .map(|(label, value)| vertical_bar(label, value, theme, font))
           .collect::<Vec<_>>()
    )
   .spacing(12)
   .width(Length::Fill)
   .height(Length::Fill)
   .padding(8);

    container(
        container(
            stack![
                container(
                    container(bars_row)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .padding(iced::padding::top(25))
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
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