use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;

/// Command line arguments for the Pomodoro timer
#[derive(Parser, Debug)]
#[command(name = "rustdoro")]
#[command(about = "A terminal-based Pomodoro timer written in Rust")]
pub struct CliArgs {
    /// Work session duration in minutes
    #[arg(short = 'w', long = "work-duration", default_value = "25")]
    pub work_duration: u64,

    /// Short break duration in minutes
    #[arg(short = 's', long = "short-break", default_value = "5")]
    pub short_break: u64,

    /// Long break duration in minutes
    #[arg(short = 'l', long = "long-break", default_value = "15")]
    pub long_break: u64,

    /// Disable sound notifications
    #[arg(long = "no-sound")]
    pub no_sound: bool,

    /// Hide the clock display
    #[arg(long = "no-clock")]
    pub no_clock: bool,

    /// Enable focus mode (hides clock and disables sound)
    #[arg(long = "focus")]
    pub focus: bool,

    /// Number of pomodoros before a long break
    #[arg(long = "long-break-after", default_value = "4")]
    pub long_break_after: u8,
}

/// Configuration structure for the Pomodoro timer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Work session duration in minutes
    pub work_duration_minutes: u64,
    /// Short break duration in minutes
    pub short_break_duration_minutes: u64,
    /// Long break duration in minutes
    pub long_break_duration_minutes: u64,
    /// Number of pomodoros before a long break
    pub long_break_after_pomodoros: u8,
    /// Whether to enable sound notifications
    pub enable_sound: bool,
    /// Whether to hide the clock display
    pub hide_clock: bool,
    /// Focus mode (hides clock and disables sound)
    pub focus_mode: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            work_duration_minutes: 25,
            short_break_duration_minutes: 5,
            long_break_duration_minutes: 15,
            long_break_after_pomodoros: 4,
            enable_sound: true,
            hide_clock: false,
            focus_mode: false,
        }
    }
}

impl Config {
    /// Create configuration from command line arguments
    pub fn load_from_cli_args(args: CliArgs) -> Self {
        let mut config = Self {
            work_duration_minutes: args.work_duration,
            short_break_duration_minutes: args.short_break,
            long_break_duration_minutes: args.long_break,
            long_break_after_pomodoros: args.long_break_after,
            enable_sound: !args.no_sound,
            hide_clock: args.no_clock,
            focus_mode: args.focus,
        };

        // Focus mode overrides sound and clock settings
        if config.focus_mode {
            config.enable_sound = false;
            config.hide_clock = true;
        }

        config
    }

    /// Save configuration to file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let toml_string = toml::to_string_pretty(self)?;
        std::fs::write(path, toml_string)?;
        Ok(())
    }

    /// Load configuration from file
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Get the default config file path
    pub fn default_config_path() -> Result<PathBuf> {
        let mut path = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        path.push("rustdoro");
        path.push("config.toml");
        Ok(path)
    }

    /// Load configuration with fallback: file -> default
    pub fn load_with_fallback() -> Self {
        if let Ok(config_path) = Self::default_config_path() {
            if config_path.exists() {
                if let Ok(config) = Self::load_from_file(&config_path) {
                    return config;
                }
            }
        }
        Self::default()
    }
} 