# 🍅 Rustdoro - Terminal Pomodoro Timer

A modern, feature-rich terminal-based Pomodoro timer written in Rust. Rustdoro helps you stay productive using the Pomodoro Technique with a beautiful terminal user interface (TUI).

## Features

- **🎯 Full Pomodoro Technique Support**: 25-minute work sessions, 5-minute short breaks, and 15-minute long breaks
- **🖥️ Beautiful Terminal UI**: Clean, colorful interface built with ratatui
- **🔊 Audio Notifications**: Customizable sound alerts for session transitions
- **⚙️ Highly Configurable**: Command-line arguments for all timing settings
- **🎨 Color-coded Sessions**: Different colors for work, short breaks, and long breaks
- **📊 Progress Tracking**: Visual progress bar and session statistics
- **🔇 Focus Mode**: Hide clock and disable sounds for distraction-free work
- **⏸️ Pause/Resume**: Full control over your timer sessions
- **📱 Cross-platform**: Works on Windows, macOS, and Linux

## Installation

### Prerequisites

- Rust 1.70 or higher
- Cargo (comes with Rust)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/PPSherry/rustdoro.git
cd rustdoro

# Build the project
cargo build --release

# Run the application
cargo run --release
```

### Install with Cargo

```bash
cargo install rustdoro
```

## Usage

### Basic Usage

```bash
# Start with default settings (25min work, 5min short break, 15min long break)
rustdoro

# Or use cargo run during development
cargo run
```

### Command Line Options

```bash
rustdoro [OPTIONS]

Options:
  -w, --work-duration <MINUTES>    Work session duration in minutes [default: 25]
  -s, --short-break <MINUTES>      Short break duration in minutes [default: 5]
  -l, --long-break <MINUTES>       Long break duration in minutes [default: 15]
      --long-break-after <COUNT>   Number of pomodoros before long break [default: 4]
      --no-sound                   Disable sound notifications
      --no-clock                   Hide the clock display
      --focus                      Enable focus mode (hides clock and disables sound)
  -h, --help                       Print help information
```

### Examples

```bash
# Custom work and break durations
rustdoro --work-duration 30 --short-break 10 --long-break 20

# Silent mode (no sound notifications)
rustdoro --no-sound

# Focus mode (no distractions)
rustdoro --focus

# Custom pomodoro cycle (long break after 3 sessions)
rustdoro --long-break-after 3
```

### Keyboard Controls

Once the application is running, use these keyboard shortcuts:

| Key | Action |
|-----|--------|
| `P` | Start/Pause timer |
| `S` | Skip current session |
| `R` | Reset timer |
| `H` or `?` | Show/Hide help |
| `Q` or `Esc` | Quit application |


## Project Structure

```
rustdoro/
├── src/
│   ├── main.rs          # Application entry point and main loop
│   ├── timer.rs         # Timer logic and session management
│   ├── ui.rs            # Terminal user interface rendering
│   ├── config.rs        # Configuration management
│   └── notifications.rs # Audio notification handling
├── Cargo.toml           # Cargo package configuration
└── README.md            # Project documentation
```

## Configuration

Rustdoro supports configuration through:

1. **Command-line arguments** (highest priority)
2. **Configuration file**
3. **Default values**

### Configuration File

use `rustdoro --generate-config` to generate a default configuration file under home directory (~/.rustdoro.ini)

Example:
```
# Rustdoro Configuration File
# This file contains settings for the Rustdoro Pomodoro timer.
# You can edit these values to customize your experience.

[General]
no_clock = false
no_sound = false
emoji = true

[Time]
tomatoes_per_set = 4
work_minutes = 35
small_break_minutes = 5
long_break_minutes = 15

[Audio]
audio_file = "/Path/to/bell_sound.wav"
volume = 1

```

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run with debug output
RUST_LOG=debug cargo run
```

### Dependencies

- [`clap`](https://crates.io/crates/clap) - Command line argument parsing
- [`ratatui`](https://crates.io/crates/ratatui) - Terminal user interface framework
- [`crossterm`](https://crates.io/crates/crossterm) - Cross-platform terminal manipulation
- [`rodio`](https://crates.io/crates/rodio) - Audio playback
- [`tokio`](https://crates.io/crates/tokio) - Asynchronous runtime
- [`serde`](https://crates.io/crates/serde) - Serialization framework
- [`toml`](https://crates.io/crates/toml) - TOML parsing
- [`anyhow`](https://crates.io/crates/anyhow) - Error handling
- [`dirs`](https://crates.io/crates/dirs) - Directory utilities

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.


## Future Plan

- [ ] Session history and statistics
- [ ] Desktop notifications
- [ ] Themes and color schemes
- [ ] Session notes and task tracking
- [ ] Web dashboard

---

**Stay productive with Rustdoro! 🍅** 