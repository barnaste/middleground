//! Configuration file management.
//!
//! Supports loading and saving user preferences to a TOML configuration file stored in the user's
//! config directory. Provides sensible defaults when no config file exists.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// User configuration for the CLI, stored in TOML format
///
/// # Example config.toml
///
/// ```toml
/// host = "http://localhost:8080"
/// username = "alice@example.com"
/// no_color = false
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Backend host URL
    #[serde(default = "default_host")]
    pub host: String,

    /// User's email address
    #[serde(default)]
    pub username: Option<String>,

    // Disable colored output
    #[serde(default)]
    pub no_color: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: default_host(),
            username: None,
            no_color: false,
        }
    }
}

fn default_host() -> String {
    String::from("http://localhost:8080")
}

impl Config {
    /// Get the application's config directory.
    ///
    /// Returns the platform-specific application config directory:
    /// - Unix: `~/.config/mgcli/`
    /// - Windows: `%APPDATA%\mgcli\`
    /// - macOS: `~/Library/Application Support/mgcli/`
    ///
    /// # Errors
    ///
    /// Returns an error if the config directory cannot be determined.
    fn app_config_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().context("Failed to determine config directory")?;
        Ok(config_dir.join("mgcli"))
    }

    /// Get the path to the config file.
    ///
    /// Returns the platform-specific config directory path:
    /// - Unix: `~/.config/mgcli/config.toml`
    /// - Windows: `%APPDATA%\mgcli\config.toml`
    /// - macOS: `~/Library/Application Support/mgcli/config.toml`
    ///
    /// # Errors
    ///
    /// Returns an error if the config directory cannot be determined.
    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::app_config_dir()?.join("config.toml"))
    }

    /// Get the path to the config file.
    ///
    /// Returns the platform-specific config directory path:
    /// - Unix: `~/.config/mgcli/config.toml`
    /// - Windows: `%APPDATA%\mgcli\config.toml`
    /// - macOS: `~/Library/Application Support/mgcli/config.toml`
    ///
    /// # Errors
    ///
    /// Returns an error if the config directory cannot be determined.
    pub fn history_path() -> Result<PathBuf> {
        Ok(Self::app_config_dir()?.join("history.txt"))
    }

    /// Load configuration from file.
    ///
    /// If the config file doesn't exist, returns default configuration.
    /// If the file exists but cannot be parsed, returns an error.
    ///
    /// # Errors
    ///
    /// Returns an error if the config file exists but cannot be read as valid TOML.
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        Ok(config)
    }

    /// Save configuration to file.
    ///
    /// Creates the config directory if it doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Cannot create config directory
    /// * Cannot write to config file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;

        // ensure the parent directories are created prior to writing
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory: {}", parent.display())
            })?;
        }

        // create config directory if it doesn't exist
        let contents = toml::to_string_pretty(self).context("Failed to serialize config")?;

        std::fs::write(&path, contents)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;

        Ok(())
    }

    /// Display information about config file, and contents if the file exists.
    ///
    /// This information is displayed via stdout.
    /// If the path to the config file cannot be determined, prints an error message.
    pub fn display() {
        match Config::config_path() {
            Ok(path) => {
                println!("Config file location: {}", path.display());

                if path.exists() {
                    println!("Status: Exists");

                    // try and load and display
                    match Config::load() {
                        Ok(config) => {
                            println!();
                            println!("Current configuration:");
                            println!("  host     = {}", config.host);
                            println!(
                                "  username = {}",
                                config.username.unwrap_or_else(|| "None".to_string())
                            );
                            println!("  no_color = {}", config.no_color);
                        }
                        Err(e) => {
                            println!("  Failed to load config: {}", e);
                        }
                    }
                } else {
                    println!("Status: Not found (will use defaults)");
                }
            }

            Err(e) => {
                eprintln!("  Failed to determine config path: {}", e);
            }
        }
    }

    /// Merge command-line arguments with config file.
    ///
    /// Command-line arguments take precedence over config file values.
    ///
    /// # Arguments
    /// * `host` - Optiomal host from CLI args
    /// * `username` - Optiomal username from CLI args
    /// * `no_color` - Color flag from CLI args
    pub fn merge_cli_args(
        &mut self,
        host: Option<String>,
        username: Option<String>,
        no_color: Option<bool>,
    ) {
        if let Some(h) = host {
            self.host = h;
        }
        if let Some(u) = username {
            self.username = Some(u);
        }
        if let Some(c) = no_color {
            self.no_color = c;
        }
    }
}
