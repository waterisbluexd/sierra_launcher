//! Clipboard copy operations using wl-clipboard-rs

use wl_clipboard_rs::copy::{MimeType, Options, Source};

/// Copy text to clipboard using wl-clipboard-rs
pub fn copy_to_clipboard(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let opts = Options::new();
    opts.copy(
        Source::Bytes(text.as_bytes().to_vec().into()),
        MimeType::Text,
    )?;
    Ok(())
}