use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipboardContent {
    Text(String),
    FilePaths(Vec<PathBuf>),
    RichText {
        plain: String,
        html: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub content: ClipboardContent,
    pub timestamp: SystemTime,
}

impl ClipboardItem {
    pub fn new(content: ClipboardContent) -> Self {
        Self {
            content,
            timestamp: SystemTime::now(),
        }
    }

    pub fn full_content(&self) -> String {
        match &self.content {
            ClipboardContent::Text(t) => t.clone(),
            ClipboardContent::FilePaths(paths) => paths
                .iter()
                .filter_map(|p| p.to_str())
                .collect::<Vec<_>>()
                .join("\n"),
            ClipboardContent::RichText { plain, .. } => plain.clone(),
        }
    }
}
