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
}

impl App {
    /// Create a new application instance
    fn new(config: Config) -> Result<Self> {
        let timer = Timer::new(config.clone());
        let ui = AppUI::new(config.hide_clock)?;
        let notifications = NotificationManager::new(config.enable_sound)?;
        let last_session_type = timer.get_session_type();

        Ok(Self {
            timer,
            ui,
            notifications,
            last_session_type,
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
                        if input_handled {
                            return;
                        }
                    }
                } => {}
            }

            // Draw the UI
            self.ui.draw(&self.timer)?;

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
        // Play session end sound
        if let Err(e) = self.notifications.play_end_sound() {
            eprintln!("Warning: Failed to play end sound: {}", e);
        }

        // Show completion message briefly
        println!("\n🎉 Session completed! Starting next session...\n");
        tokio::time::sleep(Duration::from_millis(1000)).await;

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
    
    // Create configuration from CLI arguments
    let config = Config::load_from_cli_args(args);
    
    // Print welcome message
    println!("🍅 Welcome to Rustdoro - A Terminal Pomodoro Timer");
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

/// Print application information
fn print_app_info() {
    println!("Rustdoro v{}", env!("CARGO_PKG_VERSION"));
    println!("A terminal-based Pomodoro timer written in Rust");
    println!("Copyright (c) 2024");
    println!();
}

/// Handle panic to ensure terminal restoration
fn setup_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        use crossterm::{
            execute,
            terminal::{disable_raw_mode, LeaveAlternateScreen},
        };
        
        let _ = disable_raw_mode();
        let _ = execute!(
            std::io::stdout(),
            LeaveAlternateScreen
        );
        
        original_hook(panic_info);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = Config::default();
        assert_eq!(config.work_duration_minutes, 25);
        assert_eq!(config.short_break_duration_minutes, 5);
        assert_eq!(config.long_break_duration_minutes, 15);
        assert!(config.enable_sound);
        assert!(!config.hide_clock);
    }

    #[test]
    fn test_timer_creation() {
        let config = Config::default();
        let timer = Timer::new(config);
        assert_eq!(timer.get_pomodoros_completed(), 0);
        assert!(timer.is_stopped());
    }
}

