
use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

// Embed the shell script content directly into the binary
const LIST_PACKAGES_SCRIPT: &str = include_str!("../list_all_packages.sh");

// Configurable install keybinding (default: ctrl-i)
const INSTALL_KEYBINDING: &str = "ctrl-i";

// Install script that handles package installation based on package manager
const INSTALL_SCRIPT: &str = r#"#!/bin/bash
# Parse the selected package line and install it
LINE="$1"

# Extract package manager type and package name
if [[ "$LINE" =~ ^\[apt\][[:space:]]*([^[:space:]]+) ]]; then
    PKG="${BASH_REMATCH[1]}"
    echo "Installing $PKG via apt..."
    sudo apt install -y "$PKG"
elif [[ "$LINE" =~ ^\[flatpak-([^]]+)\][[:space:]]*([^[:space:]]+) ]]; then
    REMOTE="${BASH_REMATCH[1]}"
    PKG="${BASH_REMATCH[2]}"
    echo "Installing $PKG via flatpak..."
    if [[ "$REMOTE" == "installed" ]]; then
        echo "$PKG is already installed"
    else
        flatpak install -y "$REMOTE" "$PKG"
    fi
elif [[ "$LINE" =~ ^\[snap(-installed)?\][[:space:]]*([^[:space:]]+) ]]; then
    PKG="${BASH_REMATCH[2]}"
    if [[ -n "${BASH_REMATCH[1]}" ]]; then
        echo "$PKG is already installed"
    else
        echo "Installing $PKG via snap..."
        sudo snap install "$PKG"
    fi
elif [[ "$LINE" =~ ^\[cargo(-installed)?\][[:space:]]*([^[:space:]]+) ]]; then
    PKG="${BASH_REMATCH[2]}"
    if [[ -n "${BASH_REMATCH[1]}" ]]; then
        echo "$PKG is already installed"
    else
        echo "Installing $PKG via cargo..."
        cargo install "$PKG"
    fi
elif [[ "$LINE" =~ ^\[npm(-installed)?\][[:space:]]*([^[:space:]]+) ]]; then
    PKG="${BASH_REMATCH[2]}"
    if [[ -n "${BASH_REMATCH[1]}" ]]; then
        echo "$PKG is already installed"
    else
        echo "Installing $PKG via npm (globally)..."
        npm install -g "$PKG"
    fi
else
    echo "Could not parse package from: $LINE"
    exit 1
fi
"#;

fn main() -> Result<()> {
    // Create a temporary file for the list packages script
    let temp_dir = env::temp_dir();
    let script_path = temp_dir.join(format!("list_all_packages_{}.sh", std::process::id()));
    
    // Write the embedded script content to the temporary file
    let mut script_file = fs::File::create(&script_path)
        .context("Failed to create temporary script file")?;
    script_file.write_all(LIST_PACKAGES_SCRIPT.as_bytes())
        .context("Failed to write script content")?;
    
    // Make the script executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = script_file.metadata()?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;
    }
    
    // Create temporary install script
    let install_script_path = temp_dir.join(format!("install_package_{}.sh", std::process::id()));
    let mut install_file = fs::File::create(&install_script_path)
        .context("Failed to create install script file")?;
    install_file.write_all(INSTALL_SCRIPT.as_bytes())
        .context("Failed to write install script")?;
    
    // Make the install script executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = install_file.metadata()?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&install_script_path, perms)?;
    }
    
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
    let bind_command = format!("{}:execute-silent(bash {} {{}})+abort", INSTALL_KEYBINDING, install_script_path.display());
    
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
            "--header", &format!("apt | snap | flatpak | {} to install", INSTALL_KEYBINDING.to_uppercase()),
            "--preview-window", preview_window,
            "--bind", &bind_command,
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
    
    // Clean up the temporary script files
    let _ = fs::remove_file(&script_path);
    let _ = fs::remove_file(&install_script_path);

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
