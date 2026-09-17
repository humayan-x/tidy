# Contributing to tidy

Thank you for your interest in contributing to `tidy`! We welcome contributions of all kinds: bug fixes, performance improvements, new file categories/extensions, documentation improvements, and architectural discussions.

---

## Code of Conduct

All contributors and maintainers are expected to adhere to our [Code of Conduct](CODE_OF_CONDUCT.md). Please report unacceptable behavior to [security@tidy-project.org](mailto:security@tidy-project.org).

---

## Getting Started

### Prerequisites

- **Rust**: Version 1.75.0 or later (2021 edition).
- **Cargo**: Standard package manager included with Rust.
- **Platform**: Linux or macOS.

### Local Development Setup

1. Fork and clone the repository:
   ```bash
   git clone https://github.com/<your-username>/tidy.git
   cd tidy
   ```

2. Build debug binary:
   ```bash
   cargo build
   ```

3. Run the test suite:
   ```bash
   cargo test --all-targets
   ```

4. Check code style and linter warnings:
   ```bash
   cargo fmt -- --check
   cargo clippy --all-targets -- -D warnings
   ```

---

## Architecture Overview

`tidy` is organized into focused, modular crates under `src/`:

```
src/
├── main.rs            # Application entrypoint & CLI argument parsing (clap)
├── cli/               # CLI command handlers
│   ├── run.rs         # Single-pass organize execution
│   ├── watch.rs       # Long-running watcher daemon
│   ├── undo.rs        # Transaction rollback engine
│   ├── status.rs      # System health & ledger status reporting
│   ├── service.rs     # systemd / launchd service management
│   ├── init.rs        # Default configuration scaffolding
│   └── completions.rs # Shell completion script generation
├── core/              # Core business logic
│   ├── classifier.rs  # Extension & magic-byte classification
│   ├── normalizer.rs  # Extension cleaning & compound extension handling
│   ├── config.rs      # TOML configuration loader & validator
│   └── destination.rs # Path resolution & category target mapping
├── safety/            # Data integrity & file operations
│   ├── collision.rs   # Collision detection & incremental renaming
│   ├── mover.rs       # Atomic cross-filesystem moves & permission handling
│   ├── symlink.rs     # Symlink containment verification
│   └── ledger.rs      # SQLite ACID transaction journal
├── watch/             # Real-time event engine
│   ├── engine.rs      # notify event pipeline
│   ├── guard.rs       # In-progress download filters (.crdownload, etc.)
│   └── stability.rs   # Write-completion size & timestamp tracking
├── service/           # Platform daemon integrators
│   ├── systemd.rs     # Linux user-level unit manager
│   └── launchd.rs     # macOS launchd plist manager
└── common/            # Shared utilities, paths, logging, errors
```

---

## Guidelines for Contributions

### Safety First

`tidy` guarantees **zero accidental data loss**:
- Never overwrite existing files without explicit user action or conflict avoidance.
- Always use the `Mover` abstraction (`src/safety/mover.rs`) which handles cross-device `EXDEV` moves and preserves metadata.
- All mutating file operations **must** be recorded to the `Ledger` (`src/safety/ledger.rs`) so they can be undone with `tidy undo`.
- Always verify symlink safety before traversing (`src/safety/symlink.rs`).

### Adding File Formats & Categories

To contribute new file extensions or categories:
1. Check `src/core/classifier.rs` for existing taxonomies.
2. Add new mappings to `DEFAULT_CATEGORIES` or create appropriate categories.
3. Add corresponding unit tests in `src/core/classifier.rs` covering case sensitivity and compound extensions.

### Pull Request Workflow

1. Create a descriptive feature branch:
   ```bash
   git checkout -b feat/my-new-feature
   ```
2. Commit your changes with clear, semantic commit messages (e.g., `feat(classifier): add webp to Images`, `fix(mover): handle permission denied gracefully`).
3. Ensure all tests and linters pass:
   ```bash
   cargo test --all-targets
   cargo clippy --all-targets -- -D warnings
   cargo fmt -- --check
   ```
4. Push to your fork and submit a Pull Request against `main`. Fill out the [Pull Request Template](.github/PULL_REQUEST_TEMPLATE.md).
