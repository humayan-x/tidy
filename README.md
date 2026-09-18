# tidy

<p align="center">
  <a href="https://github.com/humayan-x/tidy/actions"><img src="https://img.shields.io/badge/build-passing-brightgreen?style=flat-square&logo=githubactions" alt="CI Build Status" /></a>
  <a href="https://www.npmjs.com/package/@humayan-x/tidy"><img src="https://img.shields.io/npm/v/@humayan-x/tidy?style=flat-square&logo=npm&color=CB3837" alt="npm version" /></a>
  <a href="https://github.com/humayan-x/tidy/releases"><img src="https://img.shields.io/badge/version-0.2.0-blue?style=flat-square" alt="Version 0.2.0" /></a>
  <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square" alt="License" /></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/rust-1.75%2B%20(2021)-orange?style=flat-square&logo=rust" alt="Rust 1.75+" /></a>
  <img src="https://img.shields.io/badge/platform-linux%20%7C%20macos-lightgrey?style=flat-square" alt="Platform" />
  <img src="https://img.shields.io/badge/memory-~3--5%20MB%20RSS-success?style=flat-square" alt="Memory RSS" />
</p>

> **A blazing-fast, zero-dependency, local-first file organizer and watcher in Rust for Linux and macOS.**

`tidy` automatically scans and organizes cluttered directories (like your `Downloads` folder) into clean, predictable category hierarchies and extension subfolders (e.g. `Documents/PDF`, `Documents/XLSX`, `Images/PNG`, `Images/JPG`). It provides guaranteed non-destructive file operations, persistent undo rollbacks via an embedded SQLite transaction ledger, real-time filesystem event watching, and background service integration (`systemd --user` on Linux and `launchd` on macOS).

---

## Key Highlights

- ⚡ **Blazing Fast**: Native compiled Rust with link-time optimization (LTO) producing a single `< 5 MB` static binary.
- 🌳 **Hierarchical Tree Organization**: Groups files cleanly by category and extension subfolders (`Documents/PDF/`, `Images/PNG/`, `Archives/ZIP/`, `Code/PY/`).
- 🔒 **Zero-Risk Non-Destructive Safety**:
  - **Predictable Collision Renaming**: Never overwrites existing files — appends standard incremental counters (`photo (1).jpg`, `archive (1).tar.gz`).
  - **Atomic Cross-Device Fallback**: Catches `EXDEV` across partition boundaries and executes staged atomic copy-and-verify before removing sources.
  - **Symlink Protection**: Never follows directory symlinks outside the target root, preventing recursive loops and escapes.
- ↩ **Transactional Undo Engine**: Every file move is recorded in a local SQLite ledger (`history.db`). Restore single or multi-run batches instantly with `tidy undo` or preview with `--dry-run`.
- 👁 **Zero-Config File Classifier**:
  - Out-of-the-box support for 70+ extensions.
  - Recognizes compound archive extensions (`.tar.gz`, `.tar.bz2`, `.tar.xz`, `.tar.zst`).
  - Magic-byte sniffing fallback (`infer`) for extensionless or unrecognized binary files.
- 🕒 **Adaptive Real-Time Watcher**:
  - Powered by kernel event monitors (`inotify` on Linux, `FSEvents` on macOS).
  - Download guards filter in-progress downloads (`.crdownload`, `.part`, `.download`, `*.aria2`).
  - Write-completion stability detector and advisory lock checks prevent premature moves.
  - Idle resident memory footprint: **~3–5 MB**.
- 🛠 **Native OS Service Management**:
  - One-command daemon control via `systemd --user` (Linux) and `launchd` (macOS).

---

## Architecture & Pipeline

```
                                  tidy Architecture Pipeline
                                  
  [ Filesystem Event ]          [ Manual Execution ]
    (inotify / FSEvents)          (tidy run / --path)
             │                             │
             ▼                             │
    ┌──────────────────┐                   │
    │  Download Guard  │ ── [In Progress] ─┼──> (Ignore: .crdownload, .part, etc.)
    └────────┬─────────┘                   │
             │                             │
             ▼                             │
    ┌──────────────────┐                   │
    │ Stability Filter │ ── [Writing] ─────┼──> (Wait for write completion & locks)
    └────────┬─────────┘                   │
             │                             │
             └──────────────┬──────────────┘
                            ▼
               ┌────────────────────────┐
               │ Extension & MIME Sniff │ ──> (Matches 70+ formats & compound exts)
               └────────────┬───────────┘
                            ▼
               ┌────────────────────────┐
               │    Target & Category   │ ──> (Config overrides / Defaults)
               └────────────┬───────────┘
                            ▼
               ┌────────────────────────┐
               │ Collision Resolution   │ ──> (Non-destructive: name (1).ext)
               └────────────┬───────────┘
                            ▼
               ┌────────────────────────┐
               │ Safe Mover (Atomic)    │ ──> (same-disk rename or staged EXDEV copy)
               └────────────┬───────────┘
                            ▼
               ┌────────────────────────┐
               │ SQLite Ledger Journal  │ ──> (Recorded into history.db for undo)
               └────────────────────────┘
```

---

## Installation

### Pre-built Script (Direct CLI Download)
```bash
curl -fsSL https://raw.githubusercontent.com/humayan-x/tidy/main/install.sh | sh
```

### Via npm or npx
Run instantly without installation via `npx`:
```bash
# Preview organization on Downloads
npx @humayan-x/tidy run --path ~/Downloads --dry-run

# Run full organize
npx @humayan-x/tidy run
```

Or install globally:
```bash
npm install -g @humayan-x/tidy
tidy run
```

> **Note for npm 10.9+ / Node 22+**:
> If npm prompts or warns about install scripts, supply `--allow-scripts` so the native binary is downloaded during installation:
> ```bash
> npm install -g --allow-scripts=@humayan-x/tidy @humayan-x/tidy
> ```

### From Source (Cargo)
Ensure you have Rust (2021 edition, 1.75+) installed:

```bash
git clone https://github.com/humayan-x/tidy.git
cd tidy
cargo build --release
```

The optimized release binary is located at `target/release/tidy`.

To install it directly into your `~/.cargo/bin`:
```bash
cargo install --path .
```

---

## CLI Usage Reference

```text
tidy [OPTIONS] <COMMAND>
```

### Command Overview

| Command | Description |
|---|---|
| [`tidy run`](#1-single-pass-organize-tidy-run) | Executes a single-pass scan and file organize cycle. |
| [`tidy watch`](#2-real-time-watcher-tidy-watch) | Starts the long-running resident daemon watching a directory. |
| [`tidy undo`](#3-persistent-undo-tidy-undo) | Rolls back operation batches using the local SQLite ledger. |
| [`tidy status`](#4-status-reporting-tidy-status) | Reports watched directories, rule summaries, database size, and daemon status. |
| [`tidy service`](#5-background-service-management-tidy-service) | Manages user-level OS background services (`systemd --user` or `launchd`). |
| [`tidy init`](#6-configuration-scaffolding-tidy-init) | Scaffolds a default, fully commented `config.toml` file. |
| [`tidy completions`](#7-shell-auto-completions-tidy-completions) | Generates shell completion scripts for `bash`, `zsh`, or `fish`. |

---

### 1. Single-Pass Organize (`tidy run`)

Organize the current working directory:
```bash
tidy run
```

Organize your `Downloads` directory with dry-run preview (no files moved):
```bash
tidy run --path ~/Downloads --dry-run
```

Organize subdirectories recursively into category folders:
```bash
tidy run --path ~/Downloads --recursive
```

Use a custom configuration file:
```bash
tidy run --config /path/to/custom_config.toml
```

---

### 2. Real-Time Watcher (`tidy watch`)

Watch your `Downloads` folder for newly added files:
```bash
tidy watch
```

Watch a custom directory recursively and organize existing files first:
```bash
tidy watch --path ~/Downloads --recursive --initial-scan
```

Configure custom event debounce window (in milliseconds):
```bash
tidy watch --debounce 3000
```

---

### 3. Persistent Undo (`tidy undo`)

Undo the most recent operation batch:
```bash
tidy undo
```

Preview an undo rollback without modifying files on disk (`--dry-run`):
```bash
tidy undo --dry-run
```

Sequentially undo the last 3 runs:
```bash
tidy undo --last 3
```

Revert a specific run by its UUID:
```bash
tidy undo --run-id 4f36402e-b183-4ce4-8cf7-1b0ee78cbfd9
```

Roll back all recorded runs in history:
```bash
tidy undo --all
```

> **Note**: If an empty category directory is left behind after an undo operation, `tidy` cleanly prunes it.

---

### 4. Status Reporting (`tidy status`)

Display active configuration, database size (including WAL/SHM), and daemon state:
```bash
tidy status
```

Output:
```text
Configuration:
  Path: /home/user/.config/tidy/config.toml
  Watch Directory: /home/user/Downloads
  Ignore Hidden: true
  Debounce: 2000ms
  Stability Tick: 500ms
  Recursive: false
  Ignored Patterns: 9

Categories & Destinations:
  Images -> Photos (12 extensions)
  Documents -> Docs (16 extensions)
  Audio -> Audio (9 extensions)
  Video -> Video (10 extensions)
  Archives -> Archives (14 extensions)
  Code -> Code (18 extensions)

Database & History:
  Ledger Path: /home/user/.local/state/tidy/history.db
  Total Database Size: 32.0 KB
  Total Moves Recorded: 42
  Total Batches: 3

Service Daemon:
  Platform: systemd --user
  Service: tidy.service
  State: active (running)
```

---

### 5. Background Service Management (`tidy service`)

Manage the resident daemon seamlessly across reboots:

#### On Linux (`systemd --user`):
```bash
# Enable and immediately start background daemon
tidy service enable

# Enable daemon watching a custom directory recursively
tidy service enable --path ~/Downloads --recursive

# Check daemon running and enabled status
tidy service status

# Follow real-time daemon logs via journalctl
tidy service logs -f

# Restart daemon
tidy service restart

# Stop and disable daemon
tidy service disable
```

#### On macOS (`launchd`):
```bash
tidy service enable
tidy service status
tidy service logs -f
tidy service disable
```

---

### 6. Configuration Scaffolding (`tidy init`)

Generate a default `config.toml` at `~/.config/tidy/config.toml` (Linux) or `~/Library/Application Support/tidy/config.toml` (macOS):
```bash
tidy init
```

Overwrite an existing configuration:
```bash
tidy init --force
```

---

### 7. Shell Auto-Completions (`tidy completions`)

Generate auto-completions for your active shell:

```bash
# Bash
tidy completions bash > ~/.local/share/bash-completion/completions/tidy

# Zsh
tidy completions zsh > ~/.zsh/completion/_tidy

# Fish
tidy completions fish > ~/.config/fish/completions/tidy.fish
```

---

## Configuration (`config.toml`)

`tidy` works zero-config out of the box. To customize categories, destinations, or ignore rules, configure `config.toml`:

```toml
[settings]
# Default directory to watch when running 'tidy watch' without --path
# watch_dir = "/home/user/Downloads"

# Whether to ignore hidden files and directories (starting with '.')
ignore_hidden = true

# Debounce interval in milliseconds
debounce_ms = 2000

# Polling interval in milliseconds for write-completion stability
stability_tick_ms = 500

# Whether to scan or watch subdirectories recursively
recursive = false

# Whether to organize files into extension-based subfolders (e.g. Documents/PDF, Images/PNG)
nest_by_extension = true

# Glob patterns to ignore
ignore_patterns = [
    "*.crdownload",
    "*.part",
    "*.download",
    "*.aria2",
    "*.tmp",
    "*.partial",
    ".DS_Store",
    "Thumbs.db",
    "*.swp",
    "*~",
]

# Custom folder names for default categories
[destinations]
Images = "Photos"
Documents = "Docs"

# User-defined custom categories
[categories]
3D = ["obj", "stl", "blend", "fbx"]
EBooks = ["epub", "mobi"]
Data = ["csv", "tsv", "parquet", "jsonl"]
```

---

## Standard Storage Specifications

| Platform | Configuration (`config.toml`) | Ledger & State (`history.db`) | Daemon Logs |
|---|---|---|---|
| **Linux** | `~/.config/tidy/config.toml` | `~/.local/state/tidy/history.db` | `journalctl --user -u tidy.service` |
| **macOS** | `~/Library/Application Support/tidy/config.toml` | `~/Library/Application Support/tidy/history.db` | `~/Library/Logs/tidy/tidy.log` |

---

## Contributing & Community

We welcome contributions from the community! Check out:
- [CONTRIBUTING.md](CONTRIBUTING.md) for local development setup, code standards, and PR workflows.
- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for community standards.
- [SECURITY.md](SECURITY.md) for security reporting guidelines.

---

## License

Dual-licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
