use iced::widget::{container, text, stack, column, scrollable, row};
use iced::{Element, Border, Color, Length};
use crate::utils::theme::Theme;
use crate::Message;

const PREVIEW_LINES: usize = 3;
const CHARS_PER_LINE: usize = 40;
const WINDOW_SIZE: usize = 7; // Virtual scrolling - only render 7 items at a time

fn create_preview_lines(content: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut exhausted = true;

    for (i, word) in content.split_whitespace().enumerate() {
        if lines.len() >= PREVIEW_LINES {
            exhausted = false;
            break;
        }

        let len = if current.is_empty() { 
            word.len() 
        } else { 
            current.len() + 1 + word.len() 
        };

        if len <= CHARS_PER_LINE {
            if !current.is_empty() { current.push(' '); }
            current.push_str(word);
        } else {
            if !current.is_empty() {
                lines.push(current.trim_end().to_string());
                current = String::new();
                if lines.len() >= PREVIEW_LINES { 
                    exhausted = false; 
                    break; 
                }
            }
            if word.len() > CHARS_PER_LINE {
                lines.push(format!("{}...", &word[..CHARS_PER_LINE.saturating_sub(3)]));
                if i < content.split_whitespace().count() - 1 { 
                    exhausted = false; 
                }
            } else { 
                current.push_str(word); 
            }
        }
    }

    if !current.is_empty() && lines.len() < PREVIEW_LINES {
        lines.push(current.trim_end().to_string());
    }

    if !exhausted && !lines.is_empty() {
        if let Some(last) = lines.last_mut() {
            if last.len() > CHARS_PER_LINE.saturating_sub(3) {
                last.truncate(CHARS_PER_LINE.saturating_sub(3));
            }
            last.push_str("...");
        }
    }

    if lines.is_empty() { 
        lines.push("(empty)".to_string()); 
    }
    lines
}

pub fn clipboard_panel_view<'a>(
    theme: &'a Theme,
    bg_with_alpha: Color,
    font: iced::Font,
    font_size: f32,
    selected_index: usize,
) -> Element<'a, Message> {
    // Get clipboard history items
    let items = crate::utils::data::search_items("");
    
    // Build list with virtual scrolling (same as app_list.rs)
    let mut list = column![].spacing(1);
    
    if items.is_empty() {
        list = list.push(
            container(
                column![
                    text("No clipboard history yet")
                        .color(theme.color6)
                        .font(font)
                        .size(font_size),
                    text("")
                        .size(font_size * 0.5),
                    text("Copy something to get started!")
                        .color(Color::from_rgba(
                            theme.color6.r,
                            theme.color6.g,
                            theme.color6.b,
                            0.5
                        ))
                        .font(font)
                        .size(font_size * 0.8),
                ]
                .spacing(4)
            )
            .padding(20)
            .width(Length::Fill)
            .center_x(Length::Fill)
        );
    } else {
        // === VIRTUAL SCROLLING LOGIC (same as app_list.rs) ===
        let mut window_start = 0;
        
        // Ensure selected item is within the visible window
        // When selection reaches the last 2 items of window, slide window down
        if selected_index >= window_start + WINDOW_SIZE - 1 {
            window_start = (selected_index + 1).saturating_sub(WINDOW_SIZE);
        }
        // When selection reaches the first 2 items of window, slide window up
        else if selected_index < window_start + 1 {
            window_start = selected_index.saturating_sub(1);
        }
        
        // Ensure window doesn't go past the end
        let max_start = items.len().saturating_sub(WINDOW_SIZE);
        if window_start > max_start {
            window_start = max_start;
        }
        
        // Calculate visible range
        let window_end = (window_start + WINDOW_SIZE).min(items.len());
        
        // Add empty line at top for spacing
        list = list.push(
            container(text(""))
                .height(Length::Fixed(8.0))
        );
        
        // === ONLY RENDER ITEMS WITHIN THE WINDOW ===
        for idx in window_start..window_end {
            let item = &items[idx];
            let content = item.full_content();
            let preview_lines = create_preview_lines(&content);
            
            let selected = idx == selected_index;
            
            let bg = if selected {
                Some(theme.color3.into())
            } else if idx % 2 == 0 {
                Some(Color::from_rgba(theme.color0.r, theme.color0.g, theme.color0.b, 0.1).into())
            } else {
                None
            };
            
            let fg = if selected {
                theme.background
            } else {
                theme.foreground
            };
            
            let number_color = if selected {
                theme.background
            } else {
                theme.color3
            };
            
            // Create multi-line preview
            let mut item_column = column![].spacing(2);
            
            // First line with number and selection indicator
            if let Some(first_line) = preview_lines.first() {
                let first_line_text = first_line.clone();
                item_column = item_column.push(
                    row![
                        text(if selected { ">>" } else { "  " })
                            .font(font)
                            .size(font_size * 0.8)
                            .color(fg),
                        text(format!("{}. ", idx + 1))
                            .font(font)
                            .size(font_size * 0.8)
                            .color(number_color),
                        text(first_line_text)
                            .font(font)
                            .size(font_size * 0.8)
                            .color(fg),
                    ]
                    .spacing(4)
                );
            }
            
            // Subsequent lines with indent
            for line in preview_lines.iter().skip(1) {
                let line_text = line.clone();
                item_column = item_column.push(
                    row![
                        text("      ")
                            .font(font)
                            .size(font_size * 0.8),
                        text(line_text)
                            .font(font)
                            .size(font_size * 0.8)
                            .color(fg),
                    ]
                    .spacing(4)
                );
            }
            
            // Add separator line
            item_column = item_column.push(
                container(text(""))
                    .width(Length::Fill)
                    .height(Length::Fixed(1.0))
                    .style(move |_| container::Style {
                        background: Some(Color::from_rgba(
                            theme.color6.r,
                            theme.color6.g,
                            theme.color6.b,
                            0.2
                        ).into()),
                        ..Default::default()
                    })
            );
            
            list = list.push(
                container(item_column)
                    .padding([6, 8])
                    .width(Length::Fill)
                    .style(move |_| container::Style {
                        background: bg,
                        border: Border::default(),
                        ..Default::default()
                    })
            );
        }
    }
    
    container(
        stack![
            container(
                container(
                container(
                    scrollable(list)
                        .width(Length::Fill)
                        .height(Length::Fill)
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(iced::padding::top(7).right(15).left(15).bottom(7))
                .style(move |_| container::Style {
                    background: None,
                    ..Default::default()
                }),
            )
            .padding(iced::padding::top(0))
            .height(Length::Fill)
            .width(Length::Fill)
            .style(move |_| container::Style {
                background: None,
                border: Border {
                    color: theme.color3,
                    width: 2.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }),
            )
            .padding(iced::padding::top(10))
            .width(Length::Fill)
            .height(Length::Fill),
            
            container(
                container(
                    text(" Clipboard ")
                        .color(theme.color6)
                        .font(font)
                        .size(font_size)
                )
                .width(Length::Shrink)
                .height(Length::Shrink)
                .style(move |_| container::Style {
                    background: Some(bg_with_alpha.into()),
                    ..Default::default()
                }),
            )
            .padding(iced::padding::left(8))
            .width(Length::Shrink)
            .height(Length::Shrink)
            .style(move |_| container::Style {
                background: None,
                ..Default::default()
            }),
        ]
    )
    .padding(iced::padding::top(218))
    .width(Length::Fill)
    .height(Length::FillPortion(1))
    .style(move |_| container::Style {
        background: None,
        ..Default::default()
    })
    .into()
}