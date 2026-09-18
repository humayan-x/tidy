//! Systemd user unit generation, options, and path resolution for Linux service management.

use std::path::PathBuf;

use directories::BaseDirs;

use crate::common::error::{Result, TidyError};

pub const SERVICE_NAME: &str = "tidy.service";

/// Configuration options for generating systemd user unit.
#[derive(Debug, Clone, Default)]
pub struct SystemdUnitOptions {
    /// Absolute or %h path to the tidy executable.
    pub executable_path: String,
    /// Optional target watch directory.
    pub watch_path: Option<PathBuf>,
    /// Optional custom configuration file.
    pub config_path: Option<PathBuf>,
    /// Whether to watch recursively.
    pub recursive: bool,
}

/// Returns the path to the user systemd unit: `~/.config/systemd/user/tidy.service`.
pub fn get_systemd_unit_path() -> Result<PathBuf> {
    let base_dirs = BaseDirs::new().ok_or_else(|| {
        TidyError::PathResolution("Failed to determine user home directory".into())
    })?;

    Ok(base_dirs
        .config_dir()
        .join("systemd")
        .join("user")
        .join(SERVICE_NAME))
}

/// Generates systemd --user service unit content from an executable path string.
pub fn generate_systemd_unit(executable_path: &str) -> String {
    generate_systemd_unit_with_options(&SystemdUnitOptions {
        executable_path: executable_path.to_string(),
        watch_path: None,
        config_path: None,
        recursive: false,
    })
}

/// Generates systemd --user service unit content with comprehensive options.
pub fn generate_systemd_unit_with_options(options: &SystemdUnitOptions) -> String {
    let mut exec_parts = Vec::new();

    // Wrap executable in quotes if it contains spaces and is not already quoted
    if options.executable_path.contains(' ') && !options.executable_path.starts_with('"') {
        exec_parts.push(format!("\"{}\"", options.executable_path));
    } else {
        exec_parts.push(options.executable_path.clone());
    }

    exec_parts.push("watch".to_string());

    if let Some(ref path) = options.watch_path {
        let p_str = path.to_string_lossy();
        if p_str.contains(' ') {
            exec_parts.push(format!("--path \"{}\"", p_str));
        } else {
            exec_parts.push(format!("--path {}", p_str));
        }
    }

    if let Some(ref cfg) = options.config_path {
        let c_str = cfg.to_string_lossy();
        if c_str.contains(' ') {
            exec_parts.push(format!("--config \"{}\"", c_str));
        } else {
            exec_parts.push(format!("--config {}", c_str));
        }
    }

    if options.recursive {
        exec_parts.push("--recursive".to_string());
    }

    let exec_start = exec_parts.join(" ");

    format!(
        r#"[Unit]
Description=tidy - Local File Organizer Daemon
Documentation=https://github.com/humayan-x/tidy

[Service]
Type=simple
ExecStart={exec_start}
Restart=on-failure
RestartSec=5s

# Logging configuration
StandardOutput=journal
StandardError=journal
SyslogIdentifier=tidy

# Resource scheduling politeness
Nice=10

# Security sandboxing
NoNewPrivileges=true

[Install]
WantedBy=default.target
"#
    )
}

/// Resolves the optimal executable path for `tidy`, preferring installed binaries
/// over temporary cargo build targets.
pub fn resolve_executable_path() -> Result<PathBuf> {
    let current_exe = std::env::current_exe().map_err(|e| {
        TidyError::Service(format!(
            "Failed to determine current executable path: {}",
            e
        ))
    })?;

    // If current executable is running from a cargo target folder, check if an installed version exists
    let is_cargo_target = current_exe
        .iter()
        .any(|comp| comp == "target" || comp == "debug" || comp == "release");

    if is_cargo_target {
        if let Some(base_dirs) = BaseDirs::new() {
            let cargo_bin = base_dirs.home_dir().join(".cargo").join("bin").join("tidy");
            if cargo_bin.exists() {
                println!(
                    "  [tidy] Notice: running from build target directory; using installed binary '{}'",
                    cargo_bin.display()
                );
                return Ok(cargo_bin);
            }
            let local_bin = base_dirs.home_dir().join(".local").join("bin").join("tidy");
            if local_bin.exists() {
                println!(
                    "  [tidy] Notice: running from build target directory; using installed binary '{}'",
                    local_bin.display()
                );
                return Ok(local_bin);
            }
        }
    }

    Ok(current_exe)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_systemd_unit_default() {
        let unit = generate_systemd_unit("/usr/local/bin/tidy");
        assert!(unit.contains("ExecStart=/usr/local/bin/tidy watch"));
        assert!(unit.contains("WantedBy=default.target"));
        assert!(!unit.contains("After=default.target")); // Ordering cycle removed
        assert!(unit.contains("Restart=on-failure"));
        assert!(unit.contains("RestartSec=5s"));
        assert!(unit.contains("StandardOutput=journal"));
        assert!(unit.contains("StandardError=journal"));
        assert!(unit.contains("SyslogIdentifier=tidy"));
        assert!(unit.contains("Nice=10"));
        assert!(unit.contains("NoNewPrivileges=true"));
    }

    #[test]
    fn test_generate_systemd_unit_with_options() {
        let options = SystemdUnitOptions {
            executable_path: "/opt/bin/tidy".to_string(),
            watch_path: Some(PathBuf::from("/home/user/Downloads")),
            config_path: Some(PathBuf::from("/home/user/.config/tidy/custom.toml")),
            recursive: true,
        };
        let unit = generate_systemd_unit_with_options(&options);
        assert!(unit.contains("ExecStart=/opt/bin/tidy watch --path /home/user/Downloads --config /home/user/.config/tidy/custom.toml --recursive"));
    }

    #[test]
    fn test_generate_systemd_unit_escaping_spaces() {
        let options = SystemdUnitOptions {
            executable_path: "/home/user with spaces/bin/tidy".to_string(),
            watch_path: Some(PathBuf::from("/home/user/My Downloads")),
            config_path: None,
            recursive: false,
        };
        let unit = generate_systemd_unit_with_options(&options);
        assert!(unit.contains(
            r#"ExecStart="/home/user with spaces/bin/tidy" watch --path "/home/user/My Downloads""#
        ));
    }

    #[test]
    fn test_get_systemd_unit_path() {
        let path = get_systemd_unit_path().expect("should resolve unit path");
        assert!(path.ends_with(".config/systemd/user/tidy.service"));
    }

    #[test]
    fn test_systemd_unit_syntax_and_sections() {
        let unit = generate_systemd_unit("/usr/bin/tidy");
        // Ensure INI headers are present
        assert!(unit.contains("[Unit]"));
        assert!(unit.contains("[Service]"));
        assert!(unit.contains("[Install]"));

        // Verify section boundaries
        let unit_idx = unit.find("[Unit]").unwrap();
        let service_idx = unit.find("[Service]").unwrap();
        let install_idx = unit.find("[Install]").unwrap();
        assert!(unit_idx < service_idx);
        assert!(service_idx < install_idx);

        // Verify no After=default.target
        assert!(!unit.contains("After=default.target"));
    }

    #[test]
    fn test_unit_file_write_and_read() {
        let temp_dir = tempfile::tempdir().expect("tempdir should create");
        let unit_file = temp_dir.path().join("tidy.service");
        let unit_content = generate_systemd_unit("/usr/local/bin/tidy");
        std::fs::write(&unit_file, &unit_content).expect("write should succeed");

        let read_back = std::fs::read_to_string(&unit_file).expect("read should succeed");
        assert_eq!(unit_content, read_back);
    }
}
