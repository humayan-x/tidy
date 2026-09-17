# @humayan-x/tidy

> **A blazing-fast, zero-dependency, local-first file organizer and watcher in Rust for Linux and macOS.**

This is the official npm wrapper for [`tidy`](https://github.com/humayan-x/tidy), providing instant access via `npx` or `npm install -g`.

---

## Quick Start with npx

No installation required:

```bash
# Organize current directory
npx @humayan-x/tidy run

# Organize Downloads directory with a dry-run preview (zero risk)
npx @humayan-x/tidy run --path ~/Downloads --dry-run

# Start real-time background watcher
npx @humayan-x/tidy watch --path ~/Downloads

# Undo the last operation
npx @humayan-x/tidy undo
```

---

## Global Installation

```bash
npm install -g @humayan-x/tidy
```

Once installed, use `tidy` directly:

```bash
tidy run
tidy watch
tidy undo
tidy status
```

---

## Key Features

- ⚡ **Native Rust Performance**: Single statically linked binary executing at native speed.
- 🔒 **Zero-Risk Non-Destructive Safety**: Never overwrites files; automatically resolves collisions with incremental counters (`file (1).ext`).
- ↩ **Full Transactional Undo**: Embedded SQLite ledger records every move so any operation can be reverted with `tidy undo`.
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
