mod config;
mod timer;
mod ui;
mod notifications;

use anyhow::Result;
use clap::Parser;
use std::time::Duration;
use tokio::time::interval;

use config::{CliArgs, Config};
use timer::{SessionType, Timer};
use ui::AppUI;
use notifications::NotificationManager;

/// Main application structure
struct App {
    timer: Timer,
    ui: AppUI,
    notifications: NotificationManager,
    last_session_type: SessionType,
    show_completion_message: bool,
}

impl App {
    /// Create a new application instance
    fn new(config: Config) -> Result<Self> {
        let timer = Timer::new(config.clone());
        let ui = AppUI::new(config.hide_clock())?;
        let notifications = NotificationManager::new(config.clone())?;
        let last_session_type = timer.get_session_type();

        Ok(Self {
            timer,
            ui,
            notifications,
            last_session_type,
            show_completion_message: false,
        })
    }

    /// Run the main application loop
    async fn run(&mut self) -> Result<()> {
        let mut tick_interval = interval(Duration::from_secs(1));
        
        loop {
            tokio::select! {
                // Handle timer ticks
                _ = tick_interval.tick() => {
                    let session_completed = self.timer.tick();
                    
                    if session_completed {
                        self.handle_session_completion().await?;
                    }
                    
                    // Check if session type changed (for notifications)
                    let current_session = self.timer.get_session_type();
                    if current_session != self.last_session_type && self.timer.is_running() {
                        self.handle_session_start(current_session).await?;
                        self.last_session_type = current_session;
                    }
                }
                
                // Handle user input (non-blocking)
                _ = async {
                    // Handle input synchronously for now
                    if let Ok(input_handled) = self.ui.handle_input(&mut self.timer) {
                        // Stop audio when user interacts with timer controls
                        if self.ui.should_stop_audio_on_input() {
                            self.notifications.stop_audio();
                            // Hide completion message when user starts interacting
                            self.show_completion_message = false;
                        }
                        
                        if input_handled {
                            return;
                        }
                    }
                } => {}
            }

            // Update UI focus based on timer state
            self.ui.update_focus_based_on_timer_state(&self.timer);
            
            // Draw the UI
            self.ui.draw(&self.timer, self.show_completion_message)?;

            // Check if we should quit
            if self.ui.should_quit {
                break;
            }

            // Small delay to prevent excessive CPU usage
            tokio::time::sleep(Duration::from_millis(16)).await;
        }

        Ok(())
    }

    /// Handle session completion
    async fn handle_session_completion(&mut self) -> Result<()> {
        // Play session end sound continuously until user interaction
        if let Err(e) = self.notifications.play_end_sound() {
            eprintln!("Warning: Failed to play end sound: {}", e);
        }

        // Show completion message in UI
        self.show_completion_message = true;
        
        // Note: Audio will continue playing until user interacts with the timer
        // The audio stopping is handled in the main loop when user input is detected

        Ok(())
    }

    /// Handle session start
    async fn handle_session_start(&mut self, session_type: SessionType) -> Result<()> {
        match session_type {
            SessionType::Work => {
                if let Err(e) = self.notifications.play_work_start_sound() {
                    eprintln!("Warning: Failed to play work start sound: {}", e);
                }
            }
            SessionType::ShortBreak | SessionType::LongBreak => {
                if let Err(e) = self.notifications.play_break_start_sound() {
                    eprintln!("Warning: Failed to play break start sound: {}", e);
                }
            }
        }

        Ok(())
    }
}

/// Main function
#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = CliArgs::parse();
    
    // Handle config file generation if requested
    if args.generate_config {
        match Config::create_sample_config() {
            Ok(()) => {
                println!("Sample configuration file created successfully!");
                println!("You can now edit the configuration file and run rustdoro again.");
                return Ok(());
            }
            Err(e) => {
                eprintln!("Failed to create sample configuration file: {}", e);
                std::process::exit(1);
            }
        }
    }
    
    // Create configuration from CLI arguments with config file support
    let config = Config::load_from_cli_args_with_config(args);
    
    // Print welcome message and current configuration
    println!("🍅 Welcome to Rustdoro - A Terminal Pomodoro Timer");
    println!("Configuration:");
    println!("  Work session: {} minutes", config.work_duration_minutes());
    println!("  Short break: {} minutes", config.short_break_duration_minutes());
    println!("  Long break: {} minutes", config.long_break_duration_minutes());
    println!("  Long break after: {} pomodoros", config.long_break_after_pomodoros());
    println!("  Sound enabled: {}", config.enable_sound());
    println!("  Hide clock: {}", config.hide_clock());
    if let Some(audio_file) = &config.audio.audio_file {
        println!("  Custom audio file: {}", audio_file);
    }
    println!("  Audio volume: {:.1}", config.audio.volume);
    println!();
    println!("Press 'h' or '?' for help once the application starts.");
    println!("Starting in 2 seconds...\n");
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Create and run the application
    let mut app = App::new(config)?;
    
    // Setup proper cleanup on exit
    let result = app.run().await;
    
    // Restore terminal state
    if let Err(e) = app.ui.restore_terminal() {
        eprintln!("Warning: Failed to restore terminal: {}", e);
    }

    // Handle any errors that occurred during execution
    match result {
        Ok(()) => {
            println!("\n👋 Thanks for using Rustdoro! Stay productive!");
        }
        Err(e) => {
            eprintln!("Application error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

// Additional helper functions for better application structure

impl Drop for App {
    fn drop(&mut self) {
        // Ensure terminal is restored even if the app panics
        let _ = self.ui.restore_terminal();
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = Config::default();
        assert_eq!(config.time.work_minutes, 25);
        assert_eq!(config.time.small_break_minutes, 5);
        assert_eq!(config.time.long_break_minutes, 15);
        assert!(!config.general.no_sound);
        assert!(!config.general.no_clock);
    }

    #[test]
    fn test_timer_creation() {
        let config = Config::default();
        let timer = Timer::new(config);
        assert_eq!(timer.get_pomodoros_completed(), 0);
        assert!(timer.is_stopped());
    }
}

