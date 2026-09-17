//! Linux user background service management using systemd --user.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use directories::BaseDirs;

use crate::cli::ServiceArgs;
use crate::common::error::{Result, TidyError};
use crate::common::paths::ensure_parent_dir;

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

/// Executes a `systemctl --user` command with diagnostic error handling.
fn run_systemctl(args: &[&str]) -> Result<std::process::Output> {
    let output = Command::new("systemctl")
        .arg("--user")
        .args(args)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                TidyError::Service(
                    "Command 'systemctl' not found. systemd is required for Linux service management."
                        .to_string(),
                )
            } else {
                TidyError::Service(format!("Failed to execute 'systemctl': {}", e))
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("Failed to connect to bus")
            || stderr.contains("No such file or directory")
        {
            return Err(TidyError::Service(format!(
                "Failed to connect to user systemd bus: {}\n\n  \
                Troubleshooting Tips:\n  \
                - Ensure systemd user session is active (check $XDG_RUNTIME_DIR).\n  \
                - For headless, SSH, or container setups, enable lingering: 'loginctl enable-linger $USER'\n  \
                - Alternatively, run 'tidy watch' directly in the foreground.",
                stderr.trim()
            )));
        }
    }

    Ok(output)
}

/// Enables and starts the user systemd service.
pub fn enable_service(args: &ServiceArgs, custom_config: Option<&Path>) -> Result<()> {
    let unit_path = get_systemd_unit_path()?;
    ensure_parent_dir(&unit_path)?;

    let exe = resolve_executable_path()?;
    let exe_str = exe.to_string_lossy().to_string();

    let options = SystemdUnitOptions {
        executable_path: exe_str,
        watch_path: args.path.clone(),
        config_path: custom_config.map(|p| p.to_path_buf()),
        recursive: args.recursive,
    };

    let unit_content = generate_systemd_unit_with_options(&options);
    fs::write(&unit_path, unit_content)?;
    println!(
        "  [tidy] Wrote systemd user unit to '{}'",
        unit_path.display()
    );

    // 1. systemctl --user daemon-reload
    let reload_out = run_systemctl(&["daemon-reload"])?;
    if !reload_out.status.success() {
        let stderr = String::from_utf8_lossy(&reload_out.stderr);
        return Err(TidyError::Service(format!(
            "systemctl --user daemon-reload failed: {}",
            stderr.trim()
        )));
    }

    // 2. systemctl --user enable --now tidy.service
    let enable_out = run_systemctl(&["enable", "--now", SERVICE_NAME])?;
    if !enable_out.status.success() {
        let stderr = String::from_utf8_lossy(&enable_out.stderr);
        return Err(TidyError::Service(format!(
            "Failed to enable and start systemd service: {}",
            stderr.trim()
        )));
    }

    println!("  ✓ Successfully enabled and started tidy background service!");
    println!("  ℹ View status with:    tidy service status");
    println!("  ℹ View live logs with: tidy service logs -f");
    Ok(())
}

/// Stops and disables the user systemd service.
pub fn disable_service() -> Result<()> {
    let unit_path = get_systemd_unit_path()?;

    // 1. Stop service
    let _ = run_systemctl(&["stop", SERVICE_NAME]);

    // 2. Disable service
    let _ = run_systemctl(&["disable", SERVICE_NAME]);

    // 3. Remove unit file if present
    if unit_path.exists() {
        let _ = fs::remove_file(&unit_path);
        println!("  [tidy] Removed systemd unit '{}'", unit_path.display());
    }

    // 4. Reload daemon
    let _ = run_systemctl(&["daemon-reload"]);

    println!("  ✓ Successfully stopped and disabled tidy background service.");
    Ok(())
}

/// Restarts the user systemd service.
pub fn restart_service() -> Result<()> {
    let restart_out = run_systemctl(&["restart", SERVICE_NAME])?;
    if !restart_out.status.success() {
        let stderr = String::from_utf8_lossy(&restart_out.stderr);
        return Err(TidyError::Service(format!(
            "Failed to restart systemd service: {}",
            stderr.trim()
        )));
    }

    println!("  ✓ Successfully restarted tidy background service.");
    Ok(())
}

/// Views background service journal logs using journalctl.
pub fn service_logs(lines: usize, follow: bool) -> Result<()> {
    let mut cmd = Command::new("journalctl");
    cmd.args([
        "--user",
        "-u",
        SERVICE_NAME,
        "-n",
        &lines.to_string(),
        "--no-pager",
    ]);
    if follow {
        cmd.arg("-f");
    }

    let status = cmd
        .status()
        .map_err(|e| TidyError::Service(format!("Failed to execute 'journalctl': {}", e)))?;

    if !status.success() {
        return Err(TidyError::Service("Failed to retrieve journal logs".into()));
    }

    Ok(())
}

/// Prints current systemd service status.
pub fn service_status() -> Result<()> {
    let unit_path = get_systemd_unit_path()?;
    if !unit_path.exists() {
        println!(
            "  [tidy] Service unit file does not exist at '{}'.",
            unit_path.display()
        );
        println!(
            "  [tidy] Service is not installed. Run 'tidy service enable' to install and start."
        );
        return Ok(());
    }

    let active = is_service_active();
    let enabled = is_service_enabled();

    println!();
    println!("  Systemd User Service: {}", SERVICE_NAME);
    println!("  Unit Path:            {}", unit_path.display());
    println!(
        "  Enabled at Login:     {}",
        if enabled { "yes" } else { "no" }
    );
    println!(
        "  Active State:         {}",
        if active {
            "active (running)"
        } else {
            "inactive (stopped)"
        }
    );
    println!();

    // Query systemctl status with --no-pager to prevent blocking pager
    let _ = Command::new("systemctl")
        .args(["--user", "status", "--no-pager", SERVICE_NAME])
        .status();

    println!();
    println!("  ℹ View real-time logs with: tidy service logs -f");
    println!("  ℹ Stop service with:        tidy service disable");
    println!();

    Ok(())
}

/// Checks whether the user systemd service is configured to start at user login.
pub fn is_service_enabled() -> bool {
    Command::new("systemctl")
        .args(["--user", "is-enabled", SERVICE_NAME])
        .output()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout.trim() == "enabled"
        })
        .unwrap_or(false)
}

/// Checks whether the user systemd service is actively running.
pub fn is_service_active() -> bool {
    Command::new("systemctl")
        .args(["--user", "is-active", SERVICE_NAME])
        .output()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout.trim() == "active"
        })
        .unwrap_or(false)
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
