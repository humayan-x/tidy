//! OS background daemon service management (systemd on Linux, launchd on macOS).

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

use crate::cli::{ServiceAction, ServiceArgs};
#[allow(unused_imports)]
use crate::common::error::{Result, TidyError};
use std::path::Path;

/// Executes a service action (enable, disable, status, restart, or logs).
pub fn execute_service(args: &ServiceArgs, custom_config: Option<&Path>) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        match args.action {
            ServiceAction::Enable => linux::enable_service(args, custom_config),
            ServiceAction::Disable => linux::disable_service(),
            ServiceAction::Status => linux::service_status(),
            ServiceAction::Restart => linux::restart_service(),
            ServiceAction::Logs => linux::service_logs(args.lines, args.follow),
        }
    }

    #[cfg(target_os = "macos")]
    {
        match args.action {
            ServiceAction::Enable => macos::enable_service(args, custom_config),
            ServiceAction::Disable => macos::disable_service(),
            ServiceAction::Status => macos::service_status(),
            ServiceAction::Restart => macos::restart_service(),
            ServiceAction::Logs => macos::service_logs(args.lines, args.follow),
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = (args, custom_config);
        Err(TidyError::Service(
            "Background service management is only supported on Linux (systemd) and macOS (launchd)".into(),
        ))
    }
}

/// Checks whether the background service is active and returns a human-readable description.
pub fn check_service_status() -> String {
    #[cfg(target_os = "linux")]
    {
        let is_active = linux::is_service_active();
        let is_enabled = linux::is_service_enabled();

        if is_active {
            "Active (systemd --user daemon running)".to_string()
        } else if is_enabled {
            "Inactive (enabled on login; start with 'tidy service restart')".to_string()
        } else {
            "Inactive (use 'tidy service enable' to start)".to_string()
        }
    }

    #[cfg(target_os = "macos")]
    {
        if macos::is_service_active() {
            "Active (launchd agent loaded)".to_string()
        } else {
            "Inactive (use 'tidy service enable' to start)".to_string()
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        "Not supported on this platform".to_string()
    }
}
