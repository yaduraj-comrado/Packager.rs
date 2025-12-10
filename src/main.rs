
use anyhow::{Context, Result};
use std::env;
use std::process::{Command, Stdio};

fn main() -> Result<()> {
    let script_path = env::current_dir()?.join("list_all_packages.sh");
    
    // Determine terminal width for preview window positioning
    let columns = std::env::var("COLUMNS")
        .ok()
        .and_then(|c| c.parse::<u32>().ok())
        .unwrap_or_else(|| {
            Command::new("tput")
                .arg("cols")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .and_then(|s| s.trim().parse::<u32>().ok())
                .unwrap_or(80)
        });

    let preview_window = if columns < 80 {
        "down:50%"
    } else {
        "right:50%"
    };

    // Start the package listing script
    let mut list_script = Command::new("bash")
        .arg(&script_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to run list_all_packages.sh script")?;

    // Launch fzf with stdin piped from the script output
    let mut fzf = Command::new("fzf")
        .args(&[
            "--exit-0",
            "--multi",
            "--no-sort",
            "--ansi",
            "--layout=reverse",
            "--exact",
            "--cycle",
            "--prompt", "Search packages> ",
            "--header", "apt | snap | flatpak",
            "--preview-window", preview_window,
        ])
        .stdin(list_script.stdout.take().context("Failed to capture script stdout")?)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to run fzf. Make sure fzf is installed.")?;

    // Wait for fzf to complete
    let status = fzf.wait().context("Failed to wait for fzf")?;

    // Clean up the background script process
    let _ = list_script.wait();

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
