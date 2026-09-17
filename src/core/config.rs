//! Configuration file parsing, serde definitions, and compiled rule engine.

use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use crate::common::error::{Result, TidyError};
use crate::common::paths::get_config_file_path;
use crate::core::taxonomy::{default_categories, default_ignore_patterns};

/// General behavioral settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Optional default directory to watch for the daemon.
    #[serde(default)]
    pub watch_dir: Option<std::path::PathBuf>,

    /// Whether to ignore hidden files and directories (names starting with '.').
    #[serde(default = "default_ignore_hidden")]
    pub ignore_hidden: bool,

    /// Glob patterns for files/directories that should be skipped.
    #[serde(default = "default_ignore_patterns")]
    pub ignore_patterns: Vec<String>,

    /// Debounce duration in milliseconds for the real-time watcher (default: 2000 ms).
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,

    /// Interval in milliseconds between file stability checks (default: 500 ms).
    #[serde(default = "default_stability_tick_ms")]
    pub stability_tick_ms: u64,

    /// Whether to scan or watch subdirectories recursively by default (default: false).
    #[serde(default)]
    pub recursive: bool,
}

fn default_ignore_hidden() -> bool {
    true
}

fn default_debounce_ms() -> u64 {
    2000
}

fn default_stability_tick_ms() -> u64 {
    500
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            watch_dir: None,
            ignore_hidden: default_ignore_hidden(),
            ignore_patterns: default_ignore_patterns(),
            debounce_ms: default_debounce_ms(),
            stability_tick_ms: default_stability_tick_ms(),
            recursive: false,
        }
    }
}

/// Generates a well-commented, complete default `config.toml` string.
pub fn generate_default_config_template() -> String {
    r#"# tidy configuration file
# Saved at ~/.config/tidy/config.toml (Linux) or ~/Library/Application Support/tidy/config.toml (macOS)

[settings]
# Default directory to monitor when running 'tidy watch' without --path
# watch_dir = "/home/user/Downloads"

# Whether to ignore hidden files and directories (starting with '.')
ignore_hidden = true

# Debounce coalescing duration in milliseconds for filesystem events (default: 2000)
debounce_ms = 2000

# Tick interval in milliseconds between file stability checks (default: 500)
stability_tick_ms = 500

# Whether to scan or watch subdirectories recursively (default: false)
recursive = false

# Glob patterns for temporary downloads, editor swap files, and system indexes to skip
ignore_patterns = [
    "*.crdownload",
    "*.part",
    "*.download",
    "*.aria2",
    "*.tmp",
    "*.partial",
    "*.downloading",
    ".DS_Store",
    "._*",
    "Thumbs.db",
    "desktop.ini",
    "*.swp",
    "*~",
    "*.bak",
]

# Custom destination folder names (optional overrides for default categories)
# Example: map Images category to a folder named "Photos" instead of "Images"
[destinations]
# Images = "Photos"
# Documents = "Docs"

# User-defined custom categories (optional)
# Uncomment to override or add custom extension groups
# [categories]
# Books = ["epub", "mobi", "pdf"]
# 3D = ["obj", "stl", "blend", "fbx"]
"#.to_string()
}

/// User configuration representation stored in `config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Behavioral settings.
    #[serde(default)]
    pub settings: Settings,

    /// Category mappings: Category name -> list of extensions.
    #[serde(default = "default_categories")]
    pub categories: BTreeMap<String, Vec<String>>,

    /// Custom target folder names: Category name -> folder name.
    /// If not specified for a category, the category name itself is used as the folder name.
    #[serde(default)]
    pub destinations: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            categories: default_categories(),
            destinations: HashMap::new(),
        }
    }
}

impl Config {
    /// Loads configuration from a custom path, or from the default platform location.
    /// If no configuration file exists, returns the default configuration without error.
    #[allow(dead_code)]
    pub fn load_or_default(custom_path: Option<&Path>) -> Result<Self> {
        let path = match custom_path {
            Some(p) => p.to_path_buf(),
            None => get_config_file_path()?,
        };

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path).map_err(|e| {
            TidyError::Config(format!(
                "Failed to read config file at '{}': {}",
                path.display(),
                e
            ))
        })?;

        let config: Config = toml::from_str(&content).map_err(|e| {
            TidyError::Config(format!(
                "Failed to parse config file at '{}': {}",
                path.display(),
                e
            ))
        })?;

        Ok(config)
    }

    /// Compiles this configuration into an optimized lookup structure for runtime classification.
    pub fn compile(&self) -> Result<CompiledRules> {
        let mut builder = GlobSetBuilder::new();
        for pattern in &self.settings.ignore_patterns {
            let glob = Glob::new(pattern).map_err(|e| {
                TidyError::Config(format!("Invalid ignore glob pattern '{}': {}", pattern, e))
            })?;
            builder.add(glob);
        }
        let glob_set = builder
            .build()
            .map_err(|e| TidyError::Config(format!("Failed to build ignore glob set: {}", e)))?;

        // Invert category map: lowercase extension -> category name
        let mut extension_to_category = HashMap::new();
        for (category, extensions) in &self.categories {
            for ext in extensions {
                extension_to_category.insert(ext.to_lowercase(), category.clone());
            }
        }

        Ok(CompiledRules {
            glob_set,
            ignore_hidden: self.settings.ignore_hidden,
            debounce_ms: self.settings.debounce_ms,
            stability_tick_ms: self.settings.stability_tick_ms,
            extension_to_category,
            destinations: self.destinations.clone(),
        })
    }
}

/// Compiled runtime rules for fast filtering and classification.
#[derive(Debug, Clone)]
pub struct CompiledRules {
    pub glob_set: GlobSet,
    pub ignore_hidden: bool,
    pub debounce_ms: u64,
    pub stability_tick_ms: u64,
    pub extension_to_category: HashMap<String, String>,
    pub destinations: HashMap<String, String>,
}

impl CompiledRules {
    /// Checks if a file or directory name should be ignored based on hidden status or glob patterns.
    pub fn is_ignored(&self, file_name: &str) -> bool {
        if self.ignore_hidden && file_name.starts_with('.') {
            return true;
        }

        self.glob_set.is_match(file_name)
    }

    /// Resolves the destination directory name for a category.
    pub fn get_destination_folder(&self, category: &str) -> String {
        self.destinations
            .get(category)
            .cloned()
            .unwrap_or_else(|| category.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_compiles() {
        let config = Config::default();
        let rules = config.compile().expect("compilation should succeed");

        assert!(rules.is_ignored(".DS_Store"));
        assert!(rules.is_ignored(".hidden_file"));
        assert!(rules.is_ignored("file.crdownload"));
        assert!(!rules.is_ignored("photo.jpg"));

        assert_eq!(
            rules.extension_to_category.get("png").map(String::as_str),
            Some("Images")
        );
        assert_eq!(
            rules
                .extension_to_category
                .get("tar.gz")
                .map(String::as_str),
            Some("Archives")
        );
    }

    #[test]
    fn test_custom_destination_override() {
        let mut config = Config::default();
        config
            .destinations
            .insert("Images".to_string(), "Pictures".to_string());

        let rules = config.compile().unwrap();
        assert_eq!(rules.get_destination_folder("Images"), "Pictures");
        assert_eq!(rules.get_destination_folder("Documents"), "Documents");
    }

    #[test]
    fn test_toml_deserialization() {
        let toml_str = r#"
        [settings]
        ignore_hidden = false
        ignore_patterns = ["*.tmp"]

        [destinations]
        Images = "Photos"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(!config.settings.ignore_hidden);
        assert_eq!(config.settings.ignore_patterns, vec!["*.tmp"]);
        assert_eq!(config.destinations.get("Images").unwrap(), "Photos");
    }
}
