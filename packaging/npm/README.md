# @humayan-x/tidy

> **A blazing-fast, local-first file organizer and watcher in Rust with zero runtime dependencies for Linux and macOS.**

This is the official npm wrapper for [`tidy`](https://github.com/humayan-x/tidy). It is a lightweight postinstall downloader that fetches the official pre-compiled native Rust binary matching your OS and architecture — providing instant convenience via `npx` or `npm install -g` with zero JavaScript overhead.

> 💡 **Direct Shell Installer**: If your environment restricts npm lifecycle scripts, install the standalone binary directly via:
> ```bash
> curl -fsSL https://raw.githubusercontent.com/humayan-x/tidy/main/install.sh | sh
> ```

---

## Quick Start with npx

No installation required:

```bash
# Organize current directory
npx @humayan-x/tidy run

# Organize Downloads directory with a dry-run preview (zero risk)
npx @humayan-x/tidy run --path ~/Downloads --dry-run

# Organize directly into category folders without subfolders (Flat mode)
npx @humayan-x/tidy run --flat

# Exclude specific folders or patterns
npx @humayan-x/tidy run --exclude 'work/**' --exclude 'drafts/**'

# Start real-time background watcher with a 5-second grace period for active edits
npx @humayan-x/tidy watch --path ~/Downloads --grace-period 5

# Undo the last operation batch (LIFO transaction rollback)
npx @humayan-x/tidy undo

# Inspect transaction database stats or prune history
npx @humayan-x/tidy history info
npx @humayan-x/tidy history prune --days 30
```

---

## Global Installation

```bash
npm install -g @humayan-x/tidy
```

> **Note for npm 10.9+ / Node 22+**:
> If npm prompts or warns about install scripts, run with `--allow-scripts` so the native binary is downloaded during installation:
> ```bash
> npm install -g --allow-scripts=@humayan-x/tidy @humayan-x/tidy
> ```

Once installed, use `tidy` directly:

```bash
tidy run
tidy run --flat
tidy watch
tidy undo
tidy history info
tidy status
```

---

## What's New in 0.2.1

- 📁 **Human-readable document subfolders**: Common document formats are now cleanly grouped into readable folders instead of raw uppercase extensions:
  - `.pdf` → `Documents/PDFs/`
  - `.doc`, `.docx` → `Documents/Word/`
  - `.xls`, `.xlsx`, `.ods` → `Documents/Excel/`
  - `.ppt`, `.pptx` → `Documents/PowerPoint/`
  - All other formats fall back to their normalized uppercase extension (e.g. `.odt` → `Documents/ODT/`, `.png` → `Images/PNG/`). Fully customizable via `[subfolders]` in `config.toml`.
- 🗂 **Flat Organization Mode (`--flat`)**: Run `tidy run --flat` or `tidy watch --flat` to organize directly into top-level category folders without extension subfolders.
- 🚫 **Path-Level Exclusions (`--exclude`)**: Exclude entire directory trees (e.g. `--exclude 'work/**'`) from being moved.
- ⏳ **Active Edit Grace Period (`--grace-period`)**: Configurable quiet duration (default: 3 seconds) preventing files actively being edited or typed into from moving mid-edit.
- 🧹 **History Ledger Management (`tidy history`)**: Inspect ledger statistics (`tidy history info`) and clean old records (`tidy history prune --days 30`) with SQLite `VACUUM`.
- 🪟 **Clean Help Output**: Guaranteed `Usage: tidy [OPTIONS] <COMMAND>` output when invoked via npm wrapper.

---

## Key Features

- ⚡ **Native Rust Performance**: Single statically linked binary executing at native speed with zero runtime dependencies.
- 🔒 **Zero-Risk Non-Destructive Safety**: Never overwrites files; automatically resolves collisions with incremental counters (`file (1).ext`).
- ↩ **Full Transactional Undo (LIFO)**: Embedded SQLite ledger records every move so operations can be reverted sequentially with `tidy undo`. Safely skips missing files (`SkippedMissing`) and recreates deleted directory trees.
- 👁 **Smart Classifier**: 70+ extensions recognized out of the box with magic-byte fallback for extensionless files.
- 🕒 **Adaptive Watcher**: Watches folders with download guards (`.crdownload`, `.part`) and stability checks to wait for large writes to complete before moving.

---

## Documentation & Repository

For complete configuration options (`config.toml`), service setup (`systemd` / `launchd`), and source code, visit:

👉 **[github.com/humayan-x/tidy](https://github.com/humayan-x/tidy)**

## License

Dual-licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))
