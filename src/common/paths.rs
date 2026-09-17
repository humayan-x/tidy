use crate::common::error::{Result, TidyError};
use directories::BaseDirs;
use std::path::{Path, PathBuf};

/// Application name used for subdirectories in standard XDG / macOS locations.
pub const APP_NAME: &str = "tidy";

/// Default configuration file name.
pub const CONFIG_FILENAME: &str = "config.toml";

/// Default SQLite database file name for operation history and undo ledger.
pub const HISTORY_DB_FILENAME: &str = "history.db";

/// Resolves the base configuration directory:
/// - Linux: `~/.config/tidy` (or `$XDG_CONFIG_HOME/tidy` or `$TIDY_CONFIG_DIR`)
/// - macOS: `~/Library/Application Support/tidy`
pub fn get_config_dir() -> Result<PathBuf> {
    if let Ok(custom) = std::env::var("TIDY_CONFIG_DIR") {
        return Ok(PathBuf::from(custom));
    }
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(xdg).join(APP_NAME));
    }

    let base_dirs = BaseDirs::new().ok_or_else(|| {
        TidyError::PathResolution(
            "Failed to determine user home or standard base directories".into(),
        )
    })?;

    Ok(base_dirs.config_dir().join(APP_NAME))
}

/// Resolves the full path to `config.toml`.
pub fn get_config_file_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join(CONFIG_FILENAME))
}

/// Resolves the application state directory for persistent data (like `history.db`):
/// - Linux: `~/.local/state/tidy` (or `$XDG_STATE_HOME/tidy` or `$TIDY_STATE_DIR`)
/// - macOS: `~/Library/Application Support/tidy`
pub fn get_state_dir() -> Result<PathBuf> {
    if let Ok(custom) = std::env::var("TIDY_STATE_DIR") {
        return Ok(PathBuf::from(custom));
    }
    if let Ok(xdg) = std::env::var("XDG_STATE_HOME") {
        return Ok(PathBuf::from(xdg).join(APP_NAME));
    }

    let base_dirs = BaseDirs::new().ok_or_else(|| {
        TidyError::PathResolution(
            "Failed to determine user home or standard base directories".into(),
        )
    })?;

    // On Linux/BSD, state_dir() points to ~/.local/state.
    // On macOS and platforms where state_dir() returns None, fall back to data_dir() (~/Library/Application Support).
    let dir = match base_dirs.state_dir() {
        Some(state) => state.join(APP_NAME),
        None => base_dirs.data_dir().join(APP_NAME),
    };

    Ok(dir)
}

/// Resolves the full path to `history.db`.
pub fn get_history_db_path() -> Result<PathBuf> {
    Ok(get_state_dir()?.join(HISTORY_DB_FILENAME))
}

/// Resolves the log directory on macOS: `~/Library/Logs/tidy`.
/// On Linux, standard daemon logging is piped directly to `systemd-journald`.
#[allow(dead_code)]
pub fn get_macos_log_dir() -> Result<PathBuf> {
    let base_dirs = BaseDirs::new().ok_or_else(|| {
        TidyError::PathResolution("Failed to determine user home directory".into())
    })?;

    Ok(base_dirs
        .home_dir()
        .join("Library")
        .join("Logs")
        .join(APP_NAME))
}

/// Ensures the parent directory of the given file path exists, creating it if needed.
#[allow(dead_code)]
pub fn ensure_parent_dir(file_path: &Path) -> Result<()> {
    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

/// Ensures the given directory path exists, creating it if needed.
#[allow(dead_code)]
pub fn ensure_dir_exists(dir_path: &Path) -> Result<()> {
    if !dir_path.exists() {
        std::fs::create_dir_all(dir_path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_dir_resolution() {
        let config_dir = get_config_dir().expect("config dir should resolve");
        assert!(config_dir.ends_with(APP_NAME));
    }

    #[test]
    fn test_config_file_path() {
        let config_file = get_config_file_path().expect("config file path should resolve");
        assert!(config_file.ends_with(format!("{}/{}", APP_NAME, CONFIG_FILENAME)));
    }

    #[test]
    fn test_state_dir_resolution() {
        let state_dir = get_state_dir().expect("state dir should resolve");
        assert!(state_dir.ends_with(APP_NAME));
    }

    #[test]
    fn test_history_db_path() {
        let db_path = get_history_db_path().expect("db path should resolve");
        assert!(db_path.ends_with(format!("{}/{}", APP_NAME, HISTORY_DB_FILENAME)));
    }

    #[test]
    fn test_ensure_dir_exists() {
        let temp_dir = tempfile::tempdir().expect("tempdir creation");
        let sub = temp_dir.path().join("a").join("b").join("c");
        assert!(!sub.exists());
        ensure_dir_exists(&sub).expect("ensure dir exists");
        assert!(sub.exists() && sub.is_dir());
    }

    #[test]
    fn test_ensure_parent_dir() {
        let temp_dir = tempfile::tempdir().expect("tempdir creation");
        let file = temp_dir.path().join("parent_test").join("file.txt");
        assert!(!file.parent().unwrap().exists());
        ensure_parent_dir(&file).expect("ensure parent dir");
        assert!(file.parent().unwrap().exists());
    }
}
