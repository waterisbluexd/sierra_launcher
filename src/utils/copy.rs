use std::process::{Command, Stdio};

pub fn copy_to_clipboard(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut child = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(text.as_bytes())?;
    }
    Ok(())
}
