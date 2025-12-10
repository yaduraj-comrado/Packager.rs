# Package Manager Fuzzy Finder - Codebase Overview

## Project Summary

A lightweight Rust-based command-line tool that provides unified fuzzy-finding interface for searching packages across multiple Linux package managers (apt, snap, and flatpak) using fzf.

## Architecture

### High-Level Design

```
User → packager_rs (Rust binary)
         ↓
         ├─> list_all_packages.sh (spawned subprocess)
         │    ├─> apt-cache pkgnames
         │    ├─> snap find/list
         │    └─> flatpak remote-ls/list
         │
         └─> fzf (spawned subprocess, receives piped stream)
              └─> User interaction & selection
```

### Key Design Decisions

1. **Streaming Architecture**: The tool uses Unix pipes to stream package data from the bash script directly to fzf in real-time, allowing immediate user interaction without waiting for all packages to load.

2. **Process Management**: Two concurrent processes are spawned:
   - `list_all_packages.sh`: Runs in background gathering packages
   - `fzf`: Runs in foreground accepting user input

3. **Adaptive UI**: Terminal width detection determines fzf preview window positioning (side vs bottom).

## Component Details

### 1. Main Binary (`src/main.rs`)

**Purpose**: Entry point that orchestrates the package search workflow.

**Key Functions**:
- Terminal width detection via `$COLUMNS` env var or `tput cols`
- Spawns `list_all_packages.sh` with piped stdout
- Spawns `fzf` with stdin connected to script's stdout
- Manages process lifecycle and exit codes

**Critical Code Patterns**:
```rust
// Streaming pipe pattern
let mut list_script = Command::new("bash")
    .stdout(Stdio::piped())
    .spawn()?;

let mut fzf = Command::new("fzf")
    .stdin(list_script.stdout.take()?)  // Direct pipe connection
    .spawn()?;
```

**Error Handling**:
- Uses `anyhow` crate for context-rich error messages
- Gracefully handles missing commands (fzf, tput)
- Cleans up child processes on exit

### 2. Package Listing Script (`list_all_packages.sh`)

**Purpose**: Asynchronously gather packages from all available package managers.

**Architecture**:
- Runs three package managers in parallel using background jobs (`&`)
- Uses `wait` to ensure all jobs complete
- Outputs unified format: `[manager] package_info`

**Package Manager Implementations**:

#### APT (Debian/Ubuntu)
```bash
apt-cache pkgnames | sed 's/^/[apt] /'
```
- **Coverage**: ~115,000 packages (complete)
- **Method**: Direct package name listing

#### Flatpak
```bash
# Remote packages
for remote in $(flatpak remotes --columns=name); do
    flatpak remote-ls "$remote" --app --columns=application,name
done

# Installed packages
flatpak list --app --columns=application,name
```
- **Coverage**: ~3,000+ packages (complete from all remotes)
- **Method**: Enumerate all remotes + installed apps

#### Snap
```bash
# Installed
snap list

# Available (via broad search)
for term in "" a-z common-categories; do
    snap find "$term"
done
```
- **Coverage**: Several thousand packages (incomplete due to API limitation)
- **Method**: Alphabet + category search (workaround for missing "list all" API)
- **Known Limitation**: Snap Store has no "list all packages" endpoint

**Performance Characteristics**:
- Parallel execution minimizes total runtime
- Output streams immediately (no buffering)
- Background jobs prevent blocking

### 3. Legacy Search Script (`search_packages.sh`)

**Status**: Kept for reference, not actively used.

**Purpose**: Query-based package search (requires search term as argument).

## Data Flow

### Startup Sequence

1. **Initialization** (main.rs)
   - Detect terminal width
   - Determine fzf layout configuration
   - Resolve script path

2. **Process Spawning**
   ```
   list_all_packages.sh → stdout (Stdio::piped)
                              ↓
                           fzf stdin (Stdio::piped)
                              ↓
                           User terminal (Stdio::inherit)
   ```

3. **Streaming Phase**
   - Packages output as discovered (non-blocking)
   - fzf updates UI in real-time
   - User can filter/search immediately

4. **Completion**
   - User selects packages (Tab for multi-select)
   - fzf outputs selections to stdout
   - Rust binary waits for fzf exit
   - Script process cleaned up

### fzf Configuration

**Critical Options**:
- `--exit-0`: Exit gracefully if no match
- `--multi`: Enable multi-select with Tab
- `--no-sort`: Preserve package manager grouping
- `--ansi`: Support colored output
- `--layout=reverse`: Top-down search box
- `--exact`: Require exact substring matches
- `--cycle`: Wrap-around navigation
- `--preview-window`: Adaptive positioning (right:50% or down:50%)

**UI Elements**:
- Prompt: "Search packages> "
- Header: "apt | snap | flatpak"

## Dependencies

### Rust Crates
- `anyhow` (1.x): Error handling with context
- `std::process`: Process spawning and management
- `std::env`: Environment variable access

### External Tools (Runtime)
- `fzf`: Fuzzy finder (required)
- `tput`: Terminal capability query (optional, fallback to 80 cols)
- `bash`: Shell script execution
- `apt-cache`: APT package listing (optional)
- `snap`: Snap package listing (optional)
- `flatpak`: Flatpak package listing (optional)

## Build & Deployment

### Build Targets
```bash
cargo build          # Debug build
cargo build --release # Optimized release build
```

### Installation Paths
- Debug: `target/debug/packager_rs`
- Release: `target/release/packager_rs`

### Deployment Considerations
- Single binary with no runtime dependencies (except external tools)
- Scripts must be in same directory as binary (uses `env::current_dir()`)
- Requires at least one package manager (apt/snap/flatpak) installed

## Testing Strategy

### Manual Testing
```bash
# Basic functionality
cargo run

# With specific filter
cargo run | grep firefox

# Multi-select test
cargo run  # Press Tab multiple times, then Enter
```

### Known Limitations
1. **Snap Coverage**: Cannot list all available snaps due to API limitation
2. **Performance**: Initial load may take 5-10 seconds for full package listing
3. **Platform**: Linux-only (apt/snap/flatpak are Linux-specific)

## Future Enhancements

### Potential Improvements
1. **Caching**: Cache package lists to speed up subsequent runs
2. **Package Info Preview**: Add preview window showing package details
3. **Direct Installation**: Pipe selected packages to install commands
4. **More Package Managers**: Add support for pacman, dnf, zypper
5. **Cross-platform**: Add support for Homebrew (macOS), Chocolatey (Windows)
6. **Async Rust**: Replace process spawning with tokio for better control

### Performance Optimizations
- Implement smart caching with TTL
- Add incremental loading indicators
- Optimize snap search coverage vs speed tradeoff

## Maintenance Notes

### When Modifying Architecture
- Keep streaming behavior (don't buffer entire output)
- Maintain fzf option compatibility with `pm sa` command
- Test with all three package managers (apt/snap/flatpak)

### When Adding Package Managers
1. Add gathering logic to `list_all_packages.sh`
2. Use background job (`&`) for parallel execution
3. Format output as `[manager] package_info`
4. Update README with new manager info

### When Updating Dependencies
- Keep `anyhow` for ergonomic error handling
- Avoid adding heavy dependencies (keep binary small)
- Test on minimal Linux systems (not just Ubuntu)

## Related Documentation
- Main README: `../README.md`
- Copilot Instructions: `copilot-instructions.md`
