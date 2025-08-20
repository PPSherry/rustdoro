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
    #[arg(short = 'l', long = "long-break", default_value = "10")]
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

    /// Path to configuration file
    #[arg(long = "path")]
    pub config_path: Option<PathBuf>,

    /// Generate a sample configuration file at the default location
    #[arg(long = "generate-config")]
    pub generate_config: bool,

    /// Audio volume (0.0 to 1.0)
    #[arg(long = "volume")]
    pub volume: Option<f32>,

    /// Custom audio file path
    #[arg(long = "audio-file")]
    pub audio_file: Option<String>,
}

/// General configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Whether to hide the clock display
    pub no_clock: bool,
    /// Whether to disable sound notifications
    pub no_sound: bool,
    /// Whether to show emoji in UI
    pub emoji: bool,
}

/// Time configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeConfig {
    /// Number of pomodoros before a long break
    pub tomatoes_per_set: u8,
    /// Work session duration in minutes
    pub work_minutes: u64,
    /// Short break duration in minutes
    pub small_break_minutes: u64,
    /// Long break duration in minutes
    pub long_break_minutes: u64,
    /// Alarm duration in seconds
    pub alarm_seconds: u64,
}

/// Audio configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Path to custom audio file for notifications
    pub audio_file: Option<String>,
    /// Audio volume (0.0 to 1.0)
    pub volume: f32,
    /// Whether to loop the audio during alarm
    pub loop_audio: bool,
}

/// Configuration structure for the Pomodoro timer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "General")]
    pub general: GeneralConfig,
    #[serde(rename = "Time")]
    pub time: TimeConfig,
    #[serde(rename = "Audio")]
    pub audio: AudioConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                no_clock: false,
                no_sound: false,
                emoji: true,
            },
            time: TimeConfig {
                tomatoes_per_set: 4,
                work_minutes: 25,
                small_break_minutes: 5,
                long_break_minutes: 10,
                alarm_seconds: 5,
            },
            audio: AudioConfig {
                audio_file: None,
                volume: 0.7,
                loop_audio: true,
            },
        }
    }
}

impl Config {
    // Convenience getters for backward compatibility
    pub fn work_duration_minutes(&self) -> u64 {
        self.time.work_minutes
    }
    
    pub fn short_break_duration_minutes(&self) -> u64 {
        self.time.small_break_minutes
    }
    
    pub fn long_break_duration_minutes(&self) -> u64 {
        self.time.long_break_minutes
    }
    
    pub fn long_break_after_pomodoros(&self) -> u8 {
        self.time.tomatoes_per_set
    }
    
    pub fn enable_sound(&self) -> bool {
        !self.general.no_sound
    }
    
    pub fn hide_clock(&self) -> bool {
        self.general.no_clock
    }

    /// Create configuration from command line arguments (legacy method)
    pub fn load_from_cli_args(args: CliArgs) -> Self {
        let mut config = Self::default();
        
        // Update based on CLI args
        config.time.work_minutes = args.work_duration;
        config.time.small_break_minutes = args.short_break;
        config.time.long_break_minutes = args.long_break;
        config.time.tomatoes_per_set = args.long_break_after;
        config.general.no_sound = args.no_sound;
        config.general.no_clock = args.no_clock;
        
        if let Some(volume) = args.volume {
            config.audio.volume = volume.clamp(0.0, 1.0);
        }
        
        if let Some(audio_file) = args.audio_file {
            config.audio.audio_file = Some(audio_file);
        }

        // Focus mode overrides sound and clock settings
        if args.focus {
            config.general.no_sound = true;
            config.general.no_clock = true;
        }

        config
    }

    /// Save configuration to file with comments
    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let toml_string = toml::to_string_pretty(self)?;
        
        // Create a nicely formatted config file with comments
        let commented_config = format!(
            "# Rustdoro Configuration File\n\
             # This file contains settings for the Rustdoro Pomodoro timer.\n\
             # You can edit these values to customize your experience.\n\
             # Similar to pydoro configuration format.\n\
             \n\
             {}\n",
            toml_string
        );
        
        std::fs::write(path, commented_config)?;
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
        let mut path = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        path.push(".rustdoro.ini");
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

    /// Load configuration from CLI args with config file support
    pub fn load_from_cli_args_with_config(args: CliArgs) -> Self {
        // First, try to load from config file (either specified or default)
        let mut config = if let Some(config_path) = &args.config_path {
            // Use specified config file
            if config_path.exists() {
                Self::load_from_file(config_path).unwrap_or_else(|e| {
                    eprintln!("Warning: Failed to load config file {:?}: {}", config_path, e);
                    eprintln!("Using default configuration...");
                    Self::default()
                })
            } else {
                eprintln!("Warning: Config file {:?} does not exist", config_path);
                eprintln!("Using default configuration...");
                Self::default()
            }
        } else {
            // Try default config file location
            Self::load_with_fallback()
        };

        // Override config with command line arguments
        // Only override if the CLI arg was explicitly provided (not default)
        if args.work_duration != 25 {
            config.time.work_minutes = args.work_duration;
        }
        if args.short_break != 5 {
            config.time.small_break_minutes = args.short_break;
        }
        if args.long_break != 10 {
            config.time.long_break_minutes = args.long_break;
        }
        if args.long_break_after != 4 {
            config.time.tomatoes_per_set = args.long_break_after;
        }
        if args.no_sound {
            config.general.no_sound = true;
        }
        if args.no_clock {
            config.general.no_clock = true;
        }
        if let Some(volume) = args.volume {
            config.audio.volume = volume.clamp(0.0, 1.0);
        }
        if let Some(audio_file) = args.audio_file {
            config.audio.audio_file = Some(audio_file);
        }
        if args.focus {
            // Focus mode overrides sound and clock settings
            config.general.no_sound = true;
            config.general.no_clock = true;
        }

        config
    }

    /// Create a sample configuration file at the default location
    pub fn create_sample_config() -> Result<()> {
        let config_path = Self::default_config_path()?;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Don't overwrite existing config
        if config_path.exists() {
            return Err(anyhow::anyhow!("Config file already exists at {:?}", config_path));
        }

        let sample_config = Self::default();
        sample_config.save_to_file(&config_path)?;
        
        println!("Created sample configuration file at: {:?}", config_path);
        Ok(())
    }

    /// Save configuration to specified path or default location
    pub fn save_config(&self, path: Option<&PathBuf>) -> Result<()> {
        let config_path = if let Some(p) = path {
            p.clone()
        } else {
            Self::default_config_path()?
        };

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        self.save_to_file(&config_path)?;
        println!("Configuration saved to: {:?}", config_path);
        Ok(())
    }
}