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
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, List, ListItem, Paragraph,
    },
    Frame, Terminal,
};
use std::io;
use crate::timer::{SessionType, Timer};

/// Menu items for the top navigation bar
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuItem {
    Start,
    Pause,
    Skip,
    Reset,
    Help,
    Exit,
}

impl MenuItem {
    /// Get all menu items in order
    pub fn all() -> Vec<MenuItem> {
        vec![
            MenuItem::Start,
            MenuItem::Pause,
            MenuItem::Skip,
            MenuItem::Reset,
            MenuItem::Help,
            MenuItem::Exit,
        ]
    }

    /// Get the display text for the menu item
    pub fn display_text(&self) -> &'static str {
        match self {
            MenuItem::Start => "Start",
            MenuItem::Pause => "Pause",
            MenuItem::Skip => "Skip",
            MenuItem::Reset => "Reset",
            MenuItem::Help => "Help",
            MenuItem::Exit => "Exit",
        }
    }

    /// Get the shortcut key for the menu item
    pub fn shortcut(&self) -> &'static str {
        match self {
            MenuItem::Start => "Space",
            MenuItem::Pause => "P",
            MenuItem::Skip => "S",
            MenuItem::Reset => "R",
            MenuItem::Help => "H",
            MenuItem::Exit => "Q",
        }
    }
}

/// UI state and configuration
pub struct AppUI {
    pub should_quit: bool,
    pub show_help: bool,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    hide_clock: bool,
    /// Currently focused menu item
    pub focused_menu_item: MenuItem,
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
            focused_menu_item: MenuItem::Start,
        })
    }

    /// Update focused menu item based on timer state
    pub fn update_focus_based_on_timer_state(&mut self, timer: &Timer) {
        // Auto-update focus based on timer state for better UX
        match self.focused_menu_item {
            MenuItem::Start if timer.is_running() => {
                self.focused_menu_item = MenuItem::Pause;
            }
            MenuItem::Pause if !timer.is_running() => {
                self.focused_menu_item = MenuItem::Start;
            }
            _ => {} // Keep current focus for other items
        }
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
        let focused_item = self.focused_menu_item;
        
        self.terminal.draw(|f| {
            render_new_ui(f, timer, hide_clock, focused_item);
            
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

    /// Move focus to the next menu item
    pub fn next_menu_item(&mut self) {
        let items = MenuItem::all();
        let current_index = items.iter().position(|&item| item == self.focused_menu_item).unwrap_or(0);
        let next_index = (current_index + 1) % items.len();
        self.focused_menu_item = items[next_index];
    }

    /// Move focus to the previous menu item
    pub fn prev_menu_item(&mut self) {
        let items = MenuItem::all();
        let current_index = items.iter().position(|&item| item == self.focused_menu_item).unwrap_or(0);
        let prev_index = if current_index == 0 { items.len() - 1 } else { current_index - 1 };
        self.focused_menu_item = items[prev_index];
    }

    /// Execute the currently focused menu item
    pub fn execute_focused_item(&mut self, timer: &mut Timer) -> bool {
        match self.focused_menu_item {
            MenuItem::Start => {
                if timer.is_stopped() || timer.is_paused() {
                    timer.toggle_pause();
                    // Update focus to pause when timer starts
                    if timer.is_running() {
                        self.focused_menu_item = MenuItem::Pause;
                    }
                }
                false
            }
            MenuItem::Pause => {
                if timer.is_running() {
                    timer.toggle_pause();
                    // Update focus to start when timer pauses
                    if timer.is_paused() {
                        self.focused_menu_item = MenuItem::Start;
                    }
                }
                false
            }
            MenuItem::Skip => {
                timer.skip_session();
                self.focused_menu_item = MenuItem::Start;
                false
            }
            MenuItem::Reset => {
                timer.reset();
                self.focused_menu_item = MenuItem::Start;
                false
            }
            MenuItem::Help => {
                self.show_help = !self.show_help;
                false
            }
            MenuItem::Exit => {
                self.should_quit = true;
                true
            }
        }
    }

    /// Process keyboard events
    fn process_key_event(&mut self, key: KeyEvent, timer: &mut Timer) -> bool {
        if self.show_help {
            // In help mode, any key closes help
            self.show_help = false;
            return false;
        }

        // Handle navigation keys
        match key.code {
            // Tab key - move to next menu item
            KeyCode::Tab => {
                self.next_menu_item();
                false
            }
            // Left arrow - move to previous menu item
            KeyCode::Left => {
                self.prev_menu_item();
                false
            }
            // Right arrow - move to next menu item
            KeyCode::Right => {
                self.next_menu_item();
                false
            }
            // Enter or Space - execute focused menu item
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.execute_focused_item(timer)
            }
            // Legacy shortcut keys (still supported)
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
                true
            }
            KeyCode::Char('p') => {
                timer.toggle_pause();
                // Update focused item based on timer state
                if timer.is_running() {
                    self.focused_menu_item = MenuItem::Pause;
                } else {
                    self.focused_menu_item = MenuItem::Start;
                }
                false
            }
            KeyCode::Char('s') => {
                timer.skip_session();
                self.focused_menu_item = MenuItem::Start;
                false
            }
            KeyCode::Char('r') => {
                timer.reset();
                self.focused_menu_item = MenuItem::Start;
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

/// Render the new single-screen UI
fn render_new_ui(f: &mut Frame, timer: &Timer, hide_clock: bool, focused_item: MenuItem) {
    let size = f.size();
    
    // Create main layout - single clean screen
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Top menu bar
            Constraint::Length(2),   // Usage hint
            Constraint::Length(3),   // Session status
            Constraint::Min(8),      // ASCII art and timer
            Constraint::Length(3),   // Statistics
        ])
        .split(size);

    render_menu_bar(f, chunks[0], focused_item, timer);
    render_usage_hint(f, chunks[1]);
    render_session_status(f, chunks[2], timer);
    render_ascii_art_center(f, chunks[3], timer, hide_clock);
    render_statistics(f, chunks[4], timer);
}

/// Render the top menu bar with focus navigation
fn render_menu_bar(f: &mut Frame, area: Rect, focused_item: MenuItem, timer: &Timer) {
    let menu_items = MenuItem::all();
    let mut spans = Vec::new();
    
    for (i, &item) in menu_items.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        
        // Determine if this item should be highlighted
        let is_focused = item == focused_item;
        
        // Special handling for Start/Pause based on timer state
        let (display_text, is_active) = match item {
            MenuItem::Start => {
                if timer.is_running() {
                    ("Start", false) // Show but inactive when running
                } else {
                    ("Start", true)
                }
            }
            MenuItem::Pause => {
                if timer.is_running() {
                    ("Pause", true)
                } else {
                    ("Pause", false) // Show but inactive when not running
                }
            }
            _ => (item.display_text(), true)
        };
        
        let style = if is_focused {
            Style::default()
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        } else if is_active {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        
        spans.push(Span::styled(format!("< {} >", display_text), style));
    }
    
    let menu_bar = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    
    f.render_widget(menu_bar, area);
}

/// Render usage hint
fn render_usage_hint(f: &mut Frame, area: Rect) {
    let hint = Paragraph::new("Press Tab/←/→ to navigate, Enter/Space to select")
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);
    
    f.render_widget(hint, area);
}

/// Render session status with colors
fn render_session_status(f: &mut Frame, area: Rect, timer: &Timer) {
    let session_type = timer.get_session_type();
    let (session_text, session_color) = match session_type {
        SessionType::Work => ("Work", Color::Green),
        SessionType::ShortBreak => ("Short Break", Color::Yellow),
        SessionType::LongBreak => ("Long Break", Color::Blue),
    };
    
    let status_text = format!("{} {}", session_type.emoji(), session_text);
    let status = Paragraph::new(status_text)
        .style(Style::default().fg(session_color).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    
    f.render_widget(status, area);
}

/// Render ASCII art center with timer
fn render_ascii_art_center(f: &mut Frame, area: Rect, timer: &Timer, hide_clock: bool) {
    let time_text = if hide_clock {
        "••:••".to_string()
    } else {
        timer.get_display_time()
    };
    
    // Create ASCII art based on progress
    let progress = timer.get_progress();
    let ascii_art = create_progress_ascii_art(progress);
    
    let session_color = match timer.get_session_type() {
        SessionType::Work => Color::Green,
        SessionType::ShortBreak => Color::Yellow,
        SessionType::LongBreak => Color::Blue,
    };
    
    // Split ASCII art into lines for individual styling
    let ascii_lines: Vec<&str> = ascii_art.split('\n').collect();
    
    // Create content with logo, ASCII art, and timer
    let mut content = vec![
        Line::from(""),
        Line::from(Span::styled(
            "🍅 R U S T D O R O 🍅", 
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
    ];
    
    // Add ASCII art lines with styling
    for line in ascii_lines {
        content.push(Line::from(Span::styled(line, Style::default().fg(session_color))));
    }
    
    // Add timer display
    content.push(Line::from(""));
    content.push(Line::from(Span::styled(
        format!("│ ⏰ {} remaining │", time_text),
        Style::default().fg(session_color).add_modifier(Modifier::BOLD)
    )));
    content.push(Line::from(""));
    
    let ascii_display = Paragraph::new(content)
        .alignment(Alignment::Center);
    
    f.render_widget(ascii_display, area);
}

/// Render statistics without borders for clean look
fn render_statistics(f: &mut Frame, area: Rect, timer: &Timer) {
    let stats_text = format!("🍅 Completed Pomodoros: {}", timer.get_pomodoros_completed());
    let stats = Paragraph::new(stats_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL));
    
    f.render_widget(stats, area);
}

/// Create ASCII art representing progress
fn create_progress_ascii_art(progress: f64) -> String {
    let segments = 8;
    let filled_segments = (progress * segments as f64) as usize;
    
    // Create a more sophisticated octagon design
    let mut art = String::new();
    
    // Top section
    art.push_str("      ╭───────╮\n");
    art.push_str("    ╱           ╲\n");
    art.push_str("   ╱             ╲\n");
    art.push_str("  ╱               ╲\n");
    
    // Middle section with progress bar
    art.push_str(" │  ");
    for i in 0..segments {
        if i < filled_segments {
            art.push('█');
        } else {
            art.push('░');
        }
    }
    art.push_str("  │\n");
    
    // Bottom section
    art.push_str("  ╲               ╱\n");
    art.push_str("   ╲             ╱\n");
    art.push_str("    ╲           ╱\n");
    art.push_str("      ╰───────╯");
    
    art
}

/// Render help popup
fn render_help_popup(f: &mut Frame) {
    let area = centered_rect(70, 80, f.size());

    let help_items = vec![
        ListItem::new("🍅 Rustdoro - Navigation Help"),
        ListItem::new(""),
        ListItem::new("Menu Navigation:"),
        ListItem::new("  [Tab] or [→]    - Next menu item"),
        ListItem::new("  [←]             - Previous menu item"),
        ListItem::new("  [Enter/Space]   - Execute focused item"),
        ListItem::new(""),
        ListItem::new("Legacy Shortcuts (still work):"),
        ListItem::new("  [P]             - Start/Pause timer"),
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