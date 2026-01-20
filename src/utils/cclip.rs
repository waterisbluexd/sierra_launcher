// src/utils/cclip.rs
use std::process::Command;
use serde_json::Value;

pub fn list_history() -> Vec<String> {
    let output = Command::new("cclip")
        .arg("list")
        .arg("--format=json")
        .output()
        .expect("cclip not found");
    
    let json: Vec<Value> = serde_json::from_slice(&output.stdout)
        .unwrap_or_default();
    
    json.iter()
        .filter_map(|v| v["data"].as_str().map(String::from))
        .collect()
}