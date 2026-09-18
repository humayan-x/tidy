# tidy

<p align="center">
  <a href="https://github.com/humayan-x/tidy/actions"><img src="https://img.shields.io/badge/build-passing-brightgreen?style=flat-square&logo=githubactions" alt="CI Build Status" /></a>
  <a href="https://www.npmjs.com/package/@humayan-x/tidy"><img src="https://img.shields.io/npm/v/@humayan-x/tidy?style=flat-square&logo=npm&color=CB3837" alt="npm version" /></a>
  <a href="https://github.com/humayan-x/tidy/releases"><img src="https://img.shields.io/badge/version-0.2.1-blue?style=flat-square" alt="Version 0.2.1" /></a>
  <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square" alt="License" /></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/rust-1.75%2B%20(2021)-orange?style=flat-square&logo=rust" alt="Rust 1.75+" /></a>
  <img src="https://img.shields.io/badge/platform-linux%20%7C%20macos-lightgrey?style=flat-square" alt="Platform" />
  <img src="https://img.shields.io/badge/memory-~3--5%20MB%20RSS-success?style=flat-square" alt="Memory RSS" />
</p>

> **A blazing-fast, local-first file organizer and watcher in Rust with zero runtime dependencies for Linux and macOS.**

`tidy` automatically scans and organizes cluttered directories (like your `Downloads` folder) into clean, predictable category hierarchies and extension subfolders (e.g. `Documents/PDFs`, `Documents/Word`, `Documents/Excel`, `Images/PNG`, `Images/JPG`). It provides guaranteed non-destructive file operations, persistent undo rollbacks via an embedded SQLite transaction ledger, real-time filesystem event watching, and background service integration (`systemd --user` on Linux and `launchd` on macOS).

---

## Key Highlights

- ⚡ **Blazing Fast**: Native compiled Rust with link-time optimization (LTO) producing a single `< 5 MB` static binary with zero runtime dependencies.
- 🌳 **Hierarchical Tree Organization**: Groups files cleanly by category and readable extension subfolders (`Documents/PDFs/`, `Documents/Word/`, `Documents/Excel/`, `Images/PNG/`, `Archives/ZIP/`, `Code/PY/`).
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

### 1. Standalone Binary (Recommended)
Installs the pre-compiled native static binary for Linux (x86_64 / aarch64) or macOS (Apple Silicon / Intel):
```bash
curl -fsSL https://raw.githubusercontent.com/humayan-x/tidy/main/install.sh | sh
```

### 2. Package Managers

#### macOS (Homebrew)
```bash
brew install humayan-x/tap/tidy
```

#### Arch Linux (AUR)
```bash
yay -S tidy-bin
```

### 3. Via npm or npx (Downloader Wrapper)
> *Note: This npm package is a postinstall downloader wrapper that fetches the official native Rust binary for your platform. It contains no JavaScript runtime overhead.*

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
```

> **Note for npm 10.9+ / Node 22+**:
> If npm prompts or warns about install scripts, supply `--allow-scripts` so the native binary is downloaded during installation:
> ```bash
> npm install -g --allow-scripts=@humayan-x/tidy @humayan-x/tidy
> ```

### 4. From Source (Cargo)
Ensure you have Rust (2021 edition, 1.75+) installed:

```bash
git clone https://github.com/humayan-x/tidy.git
cd tidy
cargo install --path .
```

---

## CLI Usage Reference

```text
Usage: tidy [OPTIONS] <COMMAND>
```

### Command Overview

| Command | Description |
|---|---|
| [`tidy run`](#1-single-pass-organize-tidy-run) | Executes a single-pass scan and file organize cycle. |
| [`tidy watch`](#2-real-time-watcher-tidy-watch) | Starts the long-running resident daemon watching a directory. |
| [`tidy undo`](#3-persistent-undo-tidy-undo) | Rolls back operation batches in LIFO order using the local SQLite ledger. |
| [`tidy history`](#4-transaction-history--pruning-tidy-history) | Inspects ledger statistics and reclaims database disk space. |
| [`tidy status`](#5-status-reporting-tidy-status) | Reports watched directories, rule summaries, database size, and daemon status. |
| [`tidy service`](#6-background-service-management-tidy-service) | Manages user-level OS background services (`systemd --user` or `launchd`). |
| [`tidy init`](#7-configuration-scaffolding-tidy-init) | Scaffolds a default, fully commented `config.toml` file. |
| [`tidy completions`](#8-shell-auto-completions-tidy-completions) | Generates shell completion scripts for `bash`, `zsh`, or `fish`. |

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

Organize directly into top-level category folders without extension subfolders (Flat mode):
```bash
tidy run --flat
```

Exclude specific directories or file patterns:
```bash
tidy run --exclude 'work/**' --exclude 'drafts/**' --exclude '*.tmp'
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

Watch in flat mode with path exclusions and a 5-second grace period for actively edited files:
```bash
tidy watch --flat --exclude 'work/**' --grace-period 5
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

`tidy` records every file operation atomically into an embedded SQLite WAL ledger (`history.db`). Undoing operations follows a strict **LIFO (Last-In, First-Out)** transaction contract to guarantee filesystem consistency.

Undo the most recent operation batch:
```bash
tidy undo
```

Preview an undo rollback without modifying files on disk (`--dry-run`):
```bash
tidy undo --dry-run
```

Sequentially undo the last 3 runs (in reverse chronological order):
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

> **Safety Contract**:
> - If an empty category directory is left behind after undoing, `tidy` cleanly prunes it.
> - If destination files were deleted or moved by the user, `tidy undo` reports them as `SkippedMissing` and continues rolling back the remaining files without error or data loss.
> - If the source parent directory was deleted, `tidy undo` automatically recreates it recursively before restoring the file.

---

### 4. Transaction History & Pruning (`tidy history`)

Inspect ledger statistics including database size, completed runs, and recorded operations:
```bash
tidy history info
```

Prune old runs and operations to reclaim disk space (executes SQLite `VACUUM`):
```bash
# Delete history records older than 30 days
tidy history prune --days 30
```

---

### 4. Status Reporting (`tidy status`)

Display active configuration, database size (including WAL/SHM), and daemon state:
```bash
tidy status
```

Output:
```text
  TIDY SYSTEM STATUS

  Configuration:
    Path:    /home/user/.config/tidy/config.toml
    Status:  Active (loaded from disk)
    Rules:   6 active categories, 2 custom destinations
    Categories: Archives, Audio, Code, Documents, Images, Video

  State & Ledger:
    Database: /home/user/.local/state/tidy/history.db
    Size:     32.0 KB

  Latest Activity:
    Run ID:    e5b38d61-8cf1-45df-bbca-bdf7bb7f975b
    Timestamp: 2026-09-18 14:32:01 UTC
    Command:   tidy run --path ~/Downloads
    Root:      /home/user/Downloads
    Status:    COMPLETED
    Files:     42 file operations recorded

  Background Service:
    Status:    Active (systemd --user: tidy.service running)
    Unit Path: /home/user/.config/systemd/user/tidy.service
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

# Whether to organize files into extension-based subfolders (e.g. Documents/PDFs, Documents/Word, Images/PNG)
# Defaults: .pdf -> PDFs, .doc/.docx -> Word, .xls/.xlsx -> Excel, .ppt/.pptx -> PowerPoint.
# All unlisted extensions normalize to uppercase (e.g. .png -> Images/PNG, .rs -> Code/RS).
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

# Custom folder names for default categories (optional)
[destinations]
Images = "Photos"
Documents = "Docs"

# Custom subfolder names for nested extension folders (optional)
# Overrides or adds to defaults (.pdf -> PDFs, .doc/.docx -> Word, .xls/.xlsx -> Excel, .ppt/.pptx -> PowerPoint)
[subfolders]
odt = "Word"
rtf = "Word"

# User-defined custom categories (optional; tidy includes 70+ default formats)
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

## Architecture & Adoption FAQ

### What happens on undo if the destination directory or file was deleted?
If you manually delete or relocate files after they were organized, `tidy undo` detects their absence and marks them as `SkippedMissing`. It safely continues rolling back all remaining files in the run without throwing errors or corrupting the ledger. If the *source* directory where files are being restored to was deleted, `tidy` automatically recreates the directory tree recursively before moving each file back.

### How does the watcher behave on network filesystems (SMB, NFS, sshfs)?
`tidy watch` monitors filesystem events via OS kernel hooks (`inotify` on Linux and `FSEvents` on macOS). Remote network protocols (CIFS/SMB, NFS, sshfs) do not broadcast server-side modifications down to the client OS kernel. Therefore, background watching on remote mounts will not detect changes made by other network clients. For network shares, we recommend running scheduled batch scans via cron or a systemd timer (e.g. `tidy run --path /mnt/nas`) instead of running a resident watch daemon.

### How do I prevent tidy from touching specific folders or working trees?
You can exclude entire directories and subtrees using the `--exclude` flag or the `exclude` setting in `config.toml`:
```bash
tidy run --exclude 'work/**' --exclude 'drafts/**'
```
Glob patterns match against both the bare filename and the relative path from the target root directory.

### What is the retention policy on `history.db`? Does it grow unbounded?
The local transaction database is stored in SQLite with Write-Ahead Logging (`WAL`). Each recorded file operation requires roughly 50 bytes of storage (~1 KB per 20 operations). For high-volume automated setups, you can monitor database size with `tidy history info` and purge runs older than a specific threshold with:
```bash
tidy history prune --days 30
```
This purges expired records and automatically executes SQLite `VACUUM` to reclaim disk pages.

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
