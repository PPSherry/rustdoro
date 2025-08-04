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
git clone https://github.com/your-username/rustdoro.git
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
  -V, --version                    Print version information
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

## Keyboard Controls

Once the application is running, use these keyboard shortcuts:

| Key | Action |
|-----|--------|
| `Space` or `P` | Start/Pause timer |
| `S` | Skip current session |
| `R` | Reset timer |
| `H` or `?` | Show/Hide help |
| `Q` or `Esc` | Quit application |

## The Pomodoro Technique

The Pomodoro Technique is a time management method developed by Francesco Cirillo in the late 1980s:

1. **Work**: Focus on a task for 25 minutes (1 Pomodoro)
2. **Short Break**: Take a 5-minute break
3. **Repeat**: Continue the work-break cycle
4. **Long Break**: After 4 Pomodoros, take a 15-30 minute break
5. **Reset**: Start the cycle again

This technique helps maintain focus and prevents burnout by providing regular breaks.

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
2. **Configuration file** (future feature)
3. **Default values** (fallback)

### Configuration File (Planned)

Future versions will support a configuration file at:
- Linux/macOS: `~/.config/rustdoro/config.toml`
- Windows: `%APPDATA%\rustdoro\config.toml`

Example configuration:
```toml
work_duration_minutes = 25
short_break_duration_minutes = 5
long_break_duration_minutes = 15
long_break_after_pomodoros = 4
enable_sound = true
hide_clock = false
focus_mode = false
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

### Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt

# Run clippy linter
cargo clippy

# Check for unused dependencies
cargo machete
```

## Dependencies

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

### Development Guidelines

1. Follow Rust naming conventions and idioms
2. Add tests for new functionality
3. Update documentation as needed
4. Run `cargo fmt` and `cargo clippy` before submitting
5. Keep commit messages clear and descriptive

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by the original Pomodoro Technique by Francesco Cirillo
- Built with the amazing Rust ecosystem
- Special thanks to the ratatui and crossterm communities

## Roadmap

- [ ] Configuration file support
- [ ] Session history and statistics
- [ ] Custom sound files
- [ ] Desktop notifications
- [ ] Themes and color schemes
- [ ] Session notes and task tracking
- [ ] Export statistics to CSV/JSON
- [ ] Web dashboard (optional)

---

**Stay productive with Rustdoro! 🍅** 