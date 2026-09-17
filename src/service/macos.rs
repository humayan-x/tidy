//! macOS LaunchAgent background service management using launchd.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use directories::BaseDirs;

use crate::cli::ServiceArgs;
use crate::common::error::{Result, TidyError};
use crate::common::paths::{ensure_dir_exists, ensure_parent_dir, get_macos_log_dir};

const PLIST_LABEL: &str = "com.user.tidy";
const PLIST_FILENAME: &str = "com.user.tidy.plist";

/// Options for generating LaunchAgent plist
#[derive(Debug, Clone, Default)]
pub struct LaunchdOptions {
    pub executable_path: String,
    pub log_dir: String,
    pub watch_path: Option<PathBuf>,
    pub config_path: Option<PathBuf>,
    pub recursive: bool,
}

/// Returns the path to the user LaunchAgent plist: `~/Library/LaunchAgents/com.user.tidy.plist`.
pub fn get_launchd_plist_path() -> Result<PathBuf> {
    let base_dirs = BaseDirs::new().ok_or_else(|| {
        TidyError::PathResolution("Failed to determine user home directory".into())
    })?;

    Ok(base_dirs
        .home_dir()
        .join("Library")
        .join("LaunchAgents")
        .join(PLIST_FILENAME))
}

/// Generates launchd plist XML content with defaults.
pub fn generate_launchd_plist(executable_path: &str, log_dir: &str) -> String {
    generate_launchd_plist_with_options(&LaunchdOptions {
        executable_path: executable_path.to_string(),
        log_dir: log_dir.to_string(),
        watch_path: None,
        config_path: None,
        recursive: false,
    })
}

/// Generates launchd plist XML content with full options.
pub fn generate_launchd_plist_with_options(options: &LaunchdOptions) -> String {
    let mut args_xml = format!(
        "        <string>{}</string>\n        <string>watch</string>\n",
        options.executable_path
    );

    if let Some(ref path) = options.watch_path {
        args_xml.push_str(&format!(
            "        <string>--path</string>\n        <string>{}</string>\n",
            path.display()
        ));
    }

    if let Some(ref cfg) = options.config_path {
        args_xml.push_str(&format!(
            "        <string>--config</string>\n        <string>{}</string>\n",
            cfg.display()
        ));
    }

    if options.recursive {
        args_xml.push_str("        <string>--recursive</string>\n");
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{}</string>
    <key>ProgramArguments</key>
    <array>
{}    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{}/tidy.log</string>
    <key>StandardErrorPath</key>
    <string>{}/tidy.err</string>
</dict>
</plist>
"#,
        PLIST_LABEL, args_xml, options.log_dir, options.log_dir
    )
}

/// Enables and starts the user LaunchAgent service.
pub fn enable_service(args: &ServiceArgs, custom_config: Option<&Path>) -> Result<()> {
    let plist_path = get_launchd_plist_path()?;
    ensure_parent_dir(&plist_path)?;

    let log_dir = get_macos_log_dir()?;
    ensure_dir_exists(&log_dir)?;

    let current_exe = std::env::current_exe().map_err(|e| {
        TidyError::Service(format!(
            "Failed to determine current executable path: {}",
            e
        ))
    })?;
    let exe_str = current_exe.to_string_lossy().to_string();
    let log_str = log_dir.to_string_lossy().to_string();

    let options = LaunchdOptions {
        executable_path: exe_str,
        log_dir: log_str,
        watch_path: args.path.clone(),
        config_path: custom_config.map(|p| p.to_path_buf()),
        recursive: args.recursive,
    };

    let plist_content = generate_launchd_plist_with_options(&options);
    fs::write(&plist_path, plist_content)?;
    println!(
        "  [tidy] Wrote LaunchAgent plist to '{}'",
        plist_path.display()
    );

    // launchctl load -w <plist_path>
    let status = Command::new("launchctl")
        .args(["load", "-w"])
        .arg(&plist_path)
        .status()
        .map_err(|e| TidyError::Service(format!("Failed to execute 'launchctl load': {}", e)))?;

    if !status.success() {
        return Err(TidyError::Service("launchctl load failed".into()));
    }

    println!("  ✓ Successfully loaded and started tidy LaunchAgent service!");
    Ok(())
}

/// Stops and disables the user LaunchAgent service.
pub fn disable_service() -> Result<()> {
    let plist_path = get_launchd_plist_path()?;

    if plist_path.exists() {
        let _ = Command::new("launchctl")
            .args(["unload", "-w"])
            .arg(&plist_path)
            .status();

        let _ = fs::remove_file(&plist_path);
        println!(
            "  [tidy] Removed LaunchAgent plist '{}'",
            plist_path.display()
        );
    }

    println!("  ✓ Successfully stopped and disabled tidy LaunchAgent service.");
    Ok(())
}

/// Prints current LaunchAgent service status.
pub fn service_status() -> Result<()> {
    let status = Command::new("launchctl")
        .args(["list", PLIST_LABEL])
        .status()
        .map_err(|e| TidyError::Service(format!("Failed to query launchctl list: {}", e)))?;

    if !status.success() {
        println!(
            "  [tidy] Service '{}' is inactive or not loaded.",
            PLIST_LABEL
        );
    }
    Ok(())
}

/// Restarts the user LaunchAgent service.
pub fn restart_service() -> Result<()> {
    let plist_path = get_launchd_plist_path()?;
    if !plist_path.exists() {
        return Err(TidyError::Service(
            "Service plist not found. Use 'tidy service enable' to install and start the service first.".into(),
        ));
    }

    let _ = Command::new("launchctl")
        .args(["unload", "-w"])
        .arg(&plist_path)
        .status();

    let status = Command::new("launchctl")
        .args(["load", "-w"])
        .arg(&plist_path)
        .status()
        .map_err(|e| TidyError::Service(format!("Failed to execute 'launchctl load': {}", e)))?;

    if !status.success() {
        return Err(TidyError::Service("launchctl load failed".into()));
    }

    println!("  ✓ Successfully restarted tidy LaunchAgent service.");
    Ok(())
}

/// Views background service logs from the log file.
pub fn service_logs(lines: usize, follow: bool) -> Result<()> {
    let log_dir = get_macos_log_dir()?;
    let log_file = log_dir.join("tidy.log");
    if !log_file.exists() {
        println!(
            "  [tidy] Log file '{}' does not exist yet.",
            log_file.display()
        );
        return Ok(());
    }

    let mut cmd = Command::new("tail");
    cmd.args(["-n", &lines.to_string()]);
    if follow {
        cmd.arg("-f");
    }
    cmd.arg(&log_file);

    cmd.status()
        .map_err(|e| TidyError::Service(format!("Failed to execute tail: {}", e)))?;
    Ok(())
}

/// Checks whether the user LaunchAgent is currently loaded.
pub fn is_service_active() -> bool {
    Command::new("launchctl")
        .args(["list", PLIST_LABEL])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_launchd_plist() {
        let plist = generate_launchd_plist("/usr/local/bin/tidy", "/Users/test/Library/Logs/tidy");
        assert!(plist.contains("<string>com.user.tidy</string>"));
        assert!(plist.contains("<string>/usr/local/bin/tidy</string>"));
        assert!(plist.contains("<string>watch</string>"));
        assert!(plist.contains("/Users/test/Library/Logs/tidy/tidy.log"));
    }

    #[test]
    fn test_generate_launchd_plist_with_options() {
        let options = LaunchdOptions {
            executable_path: "/opt/bin/tidy".to_string(),
            log_dir: "/Users/test/Library/Logs/tidy".to_string(),
            watch_path: Some(PathBuf::from("/Users/test/Downloads")),
            config_path: Some(PathBuf::from("/Users/test/.config/tidy/custom.toml")),
            recursive: true,
        };
        let plist = generate_launchd_plist_with_options(&options);
        assert!(plist.contains("<string>/opt/bin/tidy</string>"));
        assert!(plist.contains("<string>watch</string>"));
        assert!(plist.contains("<string>--path</string>"));
        assert!(plist.contains("<string>/Users/test/Downloads</string>"));
        assert!(plist.contains("<string>--config</string>"));
        assert!(plist.contains("<string>/Users/test/.config/tidy/custom.toml</string>"));
        assert!(plist.contains("<string>--recursive</string>"));
    }
}
