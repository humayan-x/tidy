//! Linux service lifecycle management and systemctl controller.

use std::fs;
use std::path::Path;
use std::process::Command;

use crate::cli::ServiceArgs;
use crate::common::error::{Result, TidyError};
use crate::common::paths::ensure_parent_dir;

use super::unit::{
    generate_systemd_unit_with_options, get_systemd_unit_path, resolve_executable_path,
    SystemdUnitOptions, SERVICE_NAME,
};

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
