# Package Manager Fuzzy Finder

A Rust-based command-line tool that opens fzf immediately for searching packages across apt, snap, and flatpak.

## Features

- **Instant TUI launch**: Opens fzf fuzzy finder immediately - no prompts, just start typing
- **Multi-source search**: Searches across three package managers:
  - **apt** (Debian/Ubuntu package manager)
  - **snap** (Universal Linux packages)
  - **flatpak** (Cross-distro application distribution)
- **Real-time fuzzy filtering**: Type to filter thousands of packages instantly
- **Multi-select support**: Select multiple packages using Tab key
- **Color-coded output**: Shows package source ([apt], [snap], [flatpak])
- **Adaptive layout**: Automatically adjusts preview window based on terminal width

## Requirements

- Rust 1.70 or later
- [fzf](https://github.com/junegunn/fzf) installed (available in most package managers)
- At least one of: apt, snap, or flatpak installed on your system

## Installation

```bash
cargo build --release
```

The binary will be available at `target/release/packager_rs`

## Usage

Simply run the program and fzf opens immediately:

```bash
cargo run
# or
./target/release/packager_rs
```

**The fzf interface will open instantly - just start typing to filter packages!**

### How It Works

1. Program runs `list_all_packages.sh` to gather packages from all sources
2. Launches fzf with the package data piped to stdin
3. Type in the search field to fuzzy-filter packages in real-time
4. Press Tab to select multiple packages
5. Press Enter to output selected packages
6. Press Esc to cancel

### fzf Controls

- **Type**: Fuzzy filter packages in real-time
- **Arrow keys**: Navigate through filtered results
- **Tab**: Select/deselect multiple items (multi-select)
- **Enter**: Confirm selection and output
- **Esc**: Cancel and exit

### Example

```bash
$ cargo run
# fzf opens immediately with all packages loaded
# Start typing "fire" → see firefox packages
# Type "chrom" → see chrome/chromium packages
# Tab to select multiple, Enter to confirm
```

## Files

- **`src/main.rs`**: Rust program that launches fzf with the package list
- **`list_all_packages.sh`**: Bash script that loads all available packages
- **`search_packages.sh`**: Legacy script for query-based searching (kept for reference)

## Performance Notes

The script loads packages from each source:

- **apt**: ~115,000 packages (complete list via `apt-cache pkgnames`)
- **flatpak**: ~3,000+ packages (complete list via `flatpak remote-ls`)
- **snap**: Several thousand packages (Snap has no "list all" API, so we search broadly)

### Snap Limitation

**Important**: The Snap Store API does not provide a "list all packages" endpoint. The script searches multiple terms (alphabet, categories) to get broad coverage, but it cannot guarantee ALL snap packages are listed. This is a limitation of Snap's design, not this tool.

If you need to search for a specific snap package that doesn't appear, you can:

1. Use `snap find <package>` directly
2. Visit <https://snapcraft.io/store>
