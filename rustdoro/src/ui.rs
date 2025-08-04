use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, Gauge, List, ListItem, Paragraph,
    },
    Frame, Terminal,
};
use std::io;
use crate::timer::{SessionType, Timer, TimerState};

/// UI state and configuration
pub struct AppUI {
    pub should_quit: bool,
    pub show_help: bool,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    hide_clock: bool,
}

impl AppUI {
    /// Initialize the terminal UI
    pub fn new(hide_clock: bool) -> Result<Self> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            should_quit: false,
            show_help: false,
            terminal,
            hide_clock,
        })
    }

    /// Restore the terminal to its original state
    pub fn restore_terminal(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    /// Draw the UI
    pub fn draw(&mut self, timer: &Timer) -> Result<()> {
        let show_help = self.show_help;
        let hide_clock = self.hide_clock;
        
        self.terminal.draw(|f| {
            render_main_ui(f, timer, hide_clock);
            
            if show_help {
                render_help_popup(f);
            }
        })?;
        Ok(())
    }

    /// Handle keyboard input
    pub fn handle_input(&mut self, timer: &mut Timer) -> Result<bool> {
        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                return Ok(self.process_key_event(key, timer));
            }
        }
        Ok(false)
    }

    /// Process keyboard events
    fn process_key_event(&mut self, key: KeyEvent, timer: &mut Timer) -> bool {
        if self.show_help {
            // In help mode, any key closes help
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('h') | KeyCode::Char('?') => {
                    self.show_help = false;
                }
                _ => {
                    self.show_help = false;
                }
            }
            return false;
        }

        // Normal mode key handling
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
                true
            }
            KeyCode::Char(' ') | KeyCode::Char('p') => {
                timer.toggle_pause();
                false
            }
            KeyCode::Char('s') => {
                timer.skip_session();
                false
            }
            KeyCode::Char('r') => {
                timer.reset();
                false
            }
            KeyCode::Char('h') | KeyCode::Char('?') => {
                self.show_help = true;
                false
            }
            _ => false,
        }
    }
}

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Render the main UI
fn render_main_ui(f: &mut Frame, timer: &Timer, hide_clock: bool) {
        let size = f.size();
        
        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(10),    // Main content
                Constraint::Length(3),  // Status bar
            ])
            .split(size);

        render_header(f, chunks[0]);
        render_main_content(f, chunks[1], timer, hide_clock);
        render_status_bar(f, chunks[2], timer);
    }

/// Render the header
fn render_header(f: &mut Frame, area: Rect) {
        let title = Paragraph::new("🍅 Rustdoro - Pomodoro Timer")
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White)),
            );
        f.render_widget(title, area);
    }

/// Render the main content area
fn render_main_content(f: &mut Frame, area: Rect, timer: &Timer, hide_clock: bool) {
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),  // Session info
                Constraint::Length(8),  // Timer display
                Constraint::Length(3),  // Progress bar
                Constraint::Min(3),     // Stats
            ])
            .split(area);

        render_session_info(f, content_chunks[0], timer);
        render_timer_display(f, content_chunks[1], timer, hide_clock);
        render_progress_bar(f, content_chunks[2], timer);
        render_stats(f, content_chunks[3], timer);
    }

/// Render session information
fn render_session_info(f: &mut Frame, area: Rect, timer: &Timer) {
        let session_type = timer.get_session_type();
        let (session_text, session_color) = match session_type {
            SessionType::Work => ("Work Session", Color::Green),
            SessionType::ShortBreak => ("Short Break", Color::Yellow),
            SessionType::LongBreak => ("Long Break", Color::Blue),
        };

        let session_info = Paragraph::new(format!(
            "{} {} - {}",
            session_type.emoji(),
            session_text,
            match timer.get_state() {
                TimerState::Running => "Running",
                TimerState::Paused => "Paused",
                TimerState::Stopped => "Ready to Start",
            }
        ))
        .style(Style::default().fg(session_color).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Current Session")
                .style(Style::default().fg(Color::White)),
        );

        f.render_widget(session_info, area);
    }

/// Render the timer display
fn render_timer_display(f: &mut Frame, area: Rect, timer: &Timer, hide_clock: bool) {
        let time_text = if hide_clock {
            "••:••".to_string()
        } else {
            timer.get_display_time()
        };

        // Create large ASCII art for time display
        let time_display = Paragraph::new(Text::from(vec![
            Line::from(""),
            Line::from(Span::styled(
                time_text,
                Style::default()
                    .fg(match timer.get_session_type() {
                        SessionType::Work => Color::Green,
                        SessionType::ShortBreak => Color::Yellow,
                        SessionType::LongBreak => Color::Blue,
                    })
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ]))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Time Remaining")
                .style(Style::default().fg(Color::White)),
        );

        f.render_widget(time_display, area);
    }

/// Render progress bar
fn render_progress_bar(f: &mut Frame, area: Rect, timer: &Timer) {
        let progress = timer.get_progress();
        let progress_percent = (progress * 100.0) as u16;

        let progress_bar = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Progress")
                    .style(Style::default().fg(Color::White)),
            )
            .gauge_style(Style::default().fg(match timer.get_session_type() {
                SessionType::Work => Color::Green,
                SessionType::ShortBreak => Color::Yellow,
                SessionType::LongBreak => Color::Blue,
            }))
            .percent(progress_percent)
            .label(format!("{}%", progress_percent));

        f.render_widget(progress_bar, area);
    }

/// Render statistics
fn render_stats(f: &mut Frame, area: Rect, timer: &Timer) {
        let stats_text = vec![
            Line::from(format!("🍅 Completed Pomodoros: {}", timer.get_pomodoros_completed())),
            Line::from(""),
            Line::from("Controls:"),
            Line::from("  [Space/P] Start/Pause  [S] Skip  [R] Reset"),
            Line::from("  [H/?] Help  [Q/Esc] Quit"),
        ];

        let stats = Paragraph::new(stats_text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Statistics & Controls")
                    .style(Style::default().fg(Color::White)),
            );

        f.render_widget(stats, area);
    }

/// Render status bar
fn render_status_bar(f: &mut Frame, area: Rect, timer: &Timer) {
        let status_text = match timer.get_state() {
            TimerState::Running => "● Running - Press [Space] to pause",
            TimerState::Paused => "⏸ Paused - Press [Space] to resume",
            TimerState::Stopped => "⏹ Stopped - Press [Space] to start",
        };

        let status_bar = Paragraph::new(status_text)
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White)),
            );

        f.render_widget(status_bar, area);
    }

/// Render help popup
fn render_help_popup(f: &mut Frame) {
        let area = centered_rect(60, 70, f.size());

        let help_items = vec![
            ListItem::new("Keyboard Shortcuts:"),
            ListItem::new(""),
            ListItem::new("  [Space] or [P]  - Start/Pause timer"),
            ListItem::new("  [S]             - Skip current session"),
            ListItem::new("  [R]             - Reset timer"),
            ListItem::new("  [H] or [?]      - Show/Hide this help"),
            ListItem::new("  [Q] or [Esc]    - Quit application"),
            ListItem::new(""),
            ListItem::new("About Pomodoro Technique:"),
            ListItem::new(""),
            ListItem::new("• Work for 25 minutes (1 Pomodoro)"),
            ListItem::new("• Take a 5-minute break"),
            ListItem::new("• After 4 Pomodoros, take a 15-minute break"),
            ListItem::new("• Repeat the cycle"),
            ListItem::new(""),
            ListItem::new("Press any key to close this help."),
        ];

        let help_list = List::new(help_items)
            .block(
                Block::default()
                    .title(" Help ")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Yellow)),
            )
            .style(Style::default().fg(Color::White));

        f.render_widget(Clear, area); // Clear the background
        f.render_widget(help_list, area);
    } 