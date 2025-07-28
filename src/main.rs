use std::{
    io::{self, Write},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
};

mod ascii_digits;
mod audio;
mod mario_animation;
use ascii_digits::create_time_display_lines;
use audio::AudioManager;
use mario_animation::MarioAnimation;

#[derive(Clone, Debug, PartialEq)]
enum TimerType {
    Work,
    Break,
}

#[derive(Clone, Debug, PartialEq)]
enum TimerMode {
    Auto,
    Manual,
}

#[derive(Clone)]
struct PomodoroSession {
    timer_type: TimerType,
    duration: Duration,
    elapsed: Duration,
    is_running: bool,
    start_time: Option<Instant>,
}

const HIGHLIGHT_COLOR: Color = Color::Rgb(0, 255, 150);
const PRIMARY_COLOR: Color = Color::Rgb(144, 255, 161); //Color::Rgb(80,250,123);

fn set_terminal_title(title: &str) {
    print!("\x1b]0;{title}\x07");
    io::stdout().flush().unwrap_or(());
}

struct PomodoroTimer {
    current_session: PomodoroSession,
    mode: TimerMode,
    completed_sessions: u32,
    show_controls_popup: bool,
    show_custom_input: bool,
    custom_input: String,
    show_mario_animation: bool,
    mario_animation: MarioAnimation,
    audio_manager: AudioManager,
    custom_work_duration: Duration,
    custom_break_duration: Duration,
}

impl PomodoroTimer {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let current_session = PomodoroSession {
            timer_type: TimerType::Work,
            duration: Duration::from_secs(25 * 60), // 25 minutes
            elapsed: Duration::from_secs(0),
            is_running: false,
            start_time: None,
        };

        Ok(PomodoroTimer {
            current_session,
            mode: TimerMode::Auto,
            completed_sessions: 0,
            show_controls_popup: false,
            show_custom_input: false,
            custom_input: String::new(),
            show_mario_animation: false,
            mario_animation: MarioAnimation::new(),
            audio_manager: AudioManager {},
            custom_work_duration: Duration::from_secs(25 * 60),
            custom_break_duration: Duration::from_secs(5 * 60),
        })
    }

    fn start_timer(&mut self, timer_type: TimerType, duration: Duration) {
        self.current_session = PomodoroSession {
            timer_type,
            duration,
            elapsed: Duration::from_secs(0),
            is_running: true,
            start_time: Some(Instant::now()),
        };
    }

    fn start_work_session(&mut self) {
        self.start_timer(TimerType::Work, self.custom_work_duration);
    }

    fn start_break_session(&mut self) {
        self.start_timer(TimerType::Break, self.custom_break_duration);
    }

    fn start_custom_session(&mut self, work_mins: u32, break_mins: Option<u32>) {
        self.custom_work_duration = Duration::from_secs((work_mins * 60) as u64);
        if let Some(break_mins) = break_mins {
            self.custom_break_duration = Duration::from_secs((break_mins * 60) as u64);
        } else {
            // Use default 5 minutes for break if not specified
            self.custom_break_duration = Duration::from_secs(5 * 60);
        }
        self.start_work_session();
    }

    fn show_custom_input_dialog(&mut self) {
        self.show_custom_input = true;
        self.custom_input.clear();
    }

    fn hide_custom_input_dialog(&mut self) {
        self.show_custom_input = false;
        self.custom_input.clear();
    }

    fn parse_and_start_custom_timer(&mut self) {
        let input = self.custom_input.trim();

        if input.is_empty() {
            self.hide_custom_input_dialog();
            return;
        }

        let result = self.parse_custom_input(input);
        match result {
            Ok((work_mins, break_mins)) => {
                self.hide_custom_input_dialog();
                self.start_custom_session(work_mins, break_mins);
            }
            Err(_) => {
                // Invalid input - keep dialog open for correction
                // In a full implementation, could show error message
            }
        }
    }

    fn parse_custom_input(&self, input: &str) -> Result<(u32, Option<u32>), String> {
        if input.contains(',') {
            // Format: "work,break" (e.g., "30,10")
            let parts: Vec<&str> = input.split(',').collect();
            if parts.len() != 2 {
                return Err("Invalid format. Use 'work,break' or just 'work'".to_string());
            }

            let work_mins = parts[0].trim().parse::<u32>().map_err(|_| "Invalid work minutes")?;
            let break_mins = parts[1].trim().parse::<u32>().map_err(|_| "Invalid break minutes")?;

            if work_mins == 0 || break_mins == 0 {
                return Err("Minutes must be greater than 0".to_string());
            }

            Ok((work_mins, Some(break_mins)))
        } else {
            // Format: "work" (e.g., "20") - use default 5min break
            let work_mins = input.parse::<u32>().map_err(|_| "Invalid work minutes")?;

            if work_mins == 0 {
                return Err("Minutes must be greater than 0".to_string());
            }

            Ok((work_mins, None)) // Will use default 5min break
        }
    }

    fn toggle_timer(&mut self) {
        if self.current_session.is_running {
            self.pause_timer();
        } else {
            self.resume_timer();
        }
    }

    fn pause_timer(&mut self) {
        if self.current_session.is_running {
            if let Some(start_time) = self.current_session.start_time {
                self.current_session.elapsed += start_time.elapsed();
            }
            self.current_session.is_running = false;
            self.current_session.start_time = None;
        }
    }

    fn resume_timer(&mut self) {
        if !self.current_session.is_running {
            self.current_session.is_running = true;
            self.current_session.start_time = Some(Instant::now());
        }
    }

    fn complete_session(&mut self) {
        self.completed_sessions += 1;
        self.play_notification();

        // Show Mario animation for work session completion
        if matches!(self.current_session.timer_type, TimerType::Work) {
            self.show_mario_animation = true;
            self.mario_animation = MarioAnimation::new();
            self.mario_animation.start();
        }

        match (&self.current_session.timer_type, &self.mode) {
            (TimerType::Work, TimerMode::Auto) => {
                // Auto mode: switch to break after work
                self.start_break_session();
            }
            (TimerType::Break, TimerMode::Auto) => {
                // Auto mode: switch to work after break
                self.start_work_session();
            }
            _ => {
                // Manual mode: stop timer
                self.current_session.is_running = false;
                self.current_session.start_time = None;
            }
        }
    }

    fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            TimerMode::Manual => TimerMode::Auto,
            TimerMode::Auto => TimerMode::Manual,
        };
    }

    fn get_timer_progress(&self) -> (Duration, Duration) {
        let current_elapsed = if self.current_session.is_running {
            if let Some(start_time) = self.current_session.start_time {
                self.current_session.elapsed + start_time.elapsed()
            } else {
                self.current_session.elapsed
            }
        } else {
            self.current_session.elapsed
        };

        (current_elapsed, self.current_session.duration)
    }

    #[allow(dead_code)]
    fn format_duration(duration: Duration) -> String {
        let total_seconds = duration.as_secs();
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{minutes:02}:{seconds:02}")
    }

    fn play_notification(&self) {
        match self.current_session.timer_type {
            TimerType::Work => self.audio_manager.play_work_complete_sound(),
            TimerType::Break => {
                // Play the combined notification + music sequence for break completion
                self.audio_manager.play_break_complete_music();
            }
        }
    }

    fn is_timer_finished(&self) -> bool {
        let (elapsed, total) = self.get_timer_progress();
        elapsed >= total
    }
}

fn ui(f: &mut Frame, timer: &PomodoroTimer) {
    // Update terminal title with countdown
    let (elapsed, total) = timer.get_timer_progress();
    let remaining = if total > elapsed { total - elapsed } else { Duration::from_secs(0) };
    let remaining_minutes = remaining.as_secs() / 60;
    let remaining_seconds = remaining.as_secs() % 60;

    let session_type = match timer.current_session.timer_type {
        TimerType::Work => "Work",
        TimerType::Break => "Break",
    };

    let title = format!("CYBER TOMATO - {session_type} {remaining_minutes:02}:{remaining_seconds:02}");
    set_terminal_title(&title);

    // If Mario animation is active, show it fullscreen
    if timer.show_mario_animation {
        let mario_canvas = timer.mario_animation.render(f.area());
        f.render_widget(mario_canvas, f.area());
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(7), // ASCII countdown (5 lines + padding)
            Constraint::Length(3), // Progress bar
            Constraint::Length(3), // Status
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("CYBER TOMATO")
        .style(Style::default().fg(PRIMARY_COLOR).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(PRIMARY_COLOR)));
    f.render_widget(title, chunks[0]);

    // ASCII Art Countdown Timer
    let (elapsed, total) = timer.get_timer_progress();
    let remaining = if total > elapsed { total - elapsed } else { Duration::from_secs(0) };

    let remaining_minutes = remaining.as_secs() / 60;
    let remaining_seconds = remaining.as_secs() % 60;
    let time_display = format!("{remaining_minutes:02}:{remaining_seconds:02}");

    // Get the session type color
    let timer_color = match timer.current_session.timer_type {
        TimerType::Work => PRIMARY_COLOR,
        TimerType::Break => Color::default(),
    };

    let countdown_lines = create_time_display_lines(&time_display, timer_color);

    let countdown_paragraph = Paragraph::new(countdown_lines).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .title("")
            .border_style(Style::default().fg(PRIMARY_COLOR)),
    );

    f.render_widget(countdown_paragraph, chunks[1]);

    // Progress bar
    let (elapsed, total) = timer.get_timer_progress();
    let progress_ratio = if total.as_secs() > 0 {
        (elapsed.as_secs() as f64 / total.as_secs() as f64).min(1.0)
    } else {
        0.0
    };

    let progress_label = Span::styled(format!("{:.1}%", progress_ratio * 100.0), Style::default().fg(Color::White));

    let progress_bar = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Progress")
                .border_style(Style::default().fg(PRIMARY_COLOR)),
        )
        .gauge_style(Style::default().fg(timer_color))
        .ratio(progress_ratio)
        .label(progress_label);
    f.render_widget(progress_bar, chunks[2]);

    // Status
    let mode_text = match timer.mode {
        TimerMode::Manual => "Manual",
        TimerMode::Auto => "Auto",
    };

    let status_text = match timer.current_session.timer_type {
        TimerType::Work => "Working",
        TimerType::Break => "On Break",
    };

    let status = Paragraph::new(vec![Line::from(vec![
        Span::raw(format!(
            "  Mode: {} | Status: {} | Done: {} | ",
            mode_text, status_text, timer.completed_sessions
        )),
        Span::styled("X", Style::default().fg(PRIMARY_COLOR).add_modifier(Modifier::BOLD)),
        Span::raw(": Help  "),
    ])])
    .alignment(Alignment::Left)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Status")
            .border_style(Style::default().fg(PRIMARY_COLOR)),
    );
    f.render_widget(status, chunks[3]);

    // Controls popup
    if timer.show_controls_popup {
        let popup_area = centered_rect(60, 60, f.area());
        f.render_widget(ratatui::widgets::Clear, popup_area);

        let controls_popup = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![Span::styled("CONTROLS", Style::default().fg(PRIMARY_COLOR).add_modifier(Modifier::BOLD))]).alignment(Alignment::Center),
            Line::from(""),
            Line::from(vec![
                Span::styled(" W  ", Style::default().fg(PRIMARY_COLOR).add_modifier(Modifier::BOLD)),
                Span::raw(" - Start 25 mins Work"),
            ]),
            Line::from(vec![
                Span::styled(" B  ", Style::default().fg(PRIMARY_COLOR).add_modifier(Modifier::BOLD)),
                Span::raw(" - Start 5 mins Break"),
            ]),
            Line::from(vec![
                Span::styled(" C  ", Style::default().fg(PRIMARY_COLOR).add_modifier(Modifier::BOLD)),
                Span::raw(" - Custom timer"),
            ]),
            Line::from(vec![
                Span::styled(" ␣/↵", Style::default().fg(PRIMARY_COLOR).add_modifier(Modifier::BOLD)),
                Span::raw(" - Pause/Resume timer"),
            ]),
            Line::from(vec![
                Span::styled(" T  ", Style::default().fg(PRIMARY_COLOR).add_modifier(Modifier::BOLD)),
                Span::raw(" - Toggle Manual/Auto mode"),
            ]),
            Line::from(vec![
                Span::styled(" M  ", Style::default().fg(PRIMARY_COLOR).add_modifier(Modifier::BOLD)),
                Span::raw(" - Mario animation"),
            ]),
            Line::from(vec![
                Span::styled(" X  ", Style::default().fg(PRIMARY_COLOR).add_modifier(Modifier::BOLD)),
                Span::raw(" - Close this popup"),
            ]),
            Line::from(vec![
                Span::styled(" Esc", Style::default().fg(PRIMARY_COLOR).add_modifier(Modifier::BOLD)),
                Span::raw(" - Exit application"),
            ]),
        ])
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help")
                .border_style(Style::default().fg(PRIMARY_COLOR)),
        );
        f.render_widget(controls_popup, popup_area);
    }

    // Custom input dialog
    if timer.show_custom_input {
        let popup_area = centered_rect(70, 50, f.area());
        f.render_widget(ratatui::widgets::Clear, popup_area);

        let input_popup = Paragraph::new(vec![
            // Line::from(""),
            // Line::from(vec![Span::styled(
            //     "CUSTOM TIMER",
            //     Style::default().fg(PRIMARY_COLOR).add_modifier(Modifier::BOLD),
            // )])
            // .alignment(Alignment::Center),
            Line::from(""),
            Line::from(vec![
                Span::raw("  Format: "),
                Span::styled("work,break", Style::default().fg(HIGHLIGHT_COLOR)),
                Span::raw(" or "),
                Span::styled("work", Style::default().fg(HIGHLIGHT_COLOR)),
            ]),
            Line::from(vec![
                Span::raw("  Examples: "),
                Span::styled("30,10", Style::default().fg(HIGHLIGHT_COLOR)),
                Span::raw(" or "),
                Span::styled("20", Style::default().fg(HIGHLIGHT_COLOR)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  Input: "),
                Span::styled(&timer.custom_input, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled("█", Style::default().fg(PRIMARY_COLOR)), // Cursor
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("↵", Style::default().fg(PRIMARY_COLOR).add_modifier(Modifier::BOLD)),
                Span::raw(" - Confirm | "),
                Span::styled("X", Style::default().fg(PRIMARY_COLOR).add_modifier(Modifier::BOLD)),
                Span::raw(" - Cancel"),
            ]),
        ])
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Custom Timer")
                .border_style(Style::default().fg(PRIMARY_COLOR))
                .title_alignment(Alignment::Center),
        );
        f.render_widget(input_popup, popup_area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::prelude::Rect) -> ratatui::prelude::Rect {
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

fn run_timer() -> Result<(), Box<dyn std::error::Error>> {
    let mut timer = match PomodoroTimer::new() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Timer initialization failed: {e}");
            eprintln!("Error details: {e:?}");
            std::process::exit(1);
        }
    };

    match enable_raw_mode() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to enable raw mode: {e}");
            return Err(e.into());
        }
    }

    let mut stdout = io::stdout();
    match execute!(stdout, EnterAlternateScreen) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to enter alternate screen: {e}");
            return Err(e.into());
        }
    }

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = match Terminal::new(backend) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to create terminal: {e}");
            return Err(e.into());
        }
    };

    let result = main_loop(&mut terminal, &mut timer);

    // Audio cleanup is now handled automatically by each individual playback

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Restore terminal title
    set_terminal_title("Terminal");

    result
}

fn main_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, timer: &mut PomodoroTimer) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| ui(f, timer))?;

        if let Ok(true) = event::poll(Duration::from_millis(100)) {
            if let Ok(Event::Key(key)) = event::read() {
                // Handle Mario animation first
                if timer.show_mario_animation {
                    if let KeyEvent {
                        code: KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' '),
                        modifiers: KeyModifiers::NONE,
                        ..
                    } = key {
                        timer.show_mario_animation = false;
                    }
                    continue;
                }

                // Handle custom input dialog
                if timer.show_custom_input {
                    match key {
                        KeyEvent {
                            code: KeyCode::Char('x'),
                            modifiers: KeyModifiers::NONE,
                            ..
                        } => {
                            timer.hide_custom_input_dialog();
                        }
                        KeyEvent {
                            code: KeyCode::Enter,
                            modifiers: KeyModifiers::NONE,
                            ..
                        } => {
                            timer.parse_and_start_custom_timer();
                        }
                        KeyEvent {
                            code: KeyCode::Backspace,
                            modifiers: KeyModifiers::NONE,
                            ..
                        } => {
                            timer.custom_input.pop();
                        }
                        KeyEvent {
                            code: KeyCode::Char(c),
                            modifiers: KeyModifiers::NONE,
                            ..
                        } => {
                            if c.is_ascii_digit() || c == ',' {
                                timer.custom_input.push(c);
                            }
                        }
                        _ => {}
                    }
                    continue;
                }

                match key {
                    KeyEvent {
                        code: KeyCode::Esc,
                        modifiers: KeyModifiers::NONE,
                        ..
                    }
                    | KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => break,

                    KeyEvent {
                        code: KeyCode::Char('w'),
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => {
                        timer.start_work_session();
                    }

                    KeyEvent {
                        code: KeyCode::Char('b'),
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => {
                        timer.start_break_session();
                    }

                    KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => {
                        timer.show_custom_input_dialog();
                    }

                    KeyEvent {
                        code: KeyCode::Enter | KeyCode::Char(' '),
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => {
                        timer.toggle_timer();
                    }

                    KeyEvent {
                        code: KeyCode::Char('t'),
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => {
                        timer.toggle_mode();
                    }

                    KeyEvent {
                        code: KeyCode::Char('x'),
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => {
                        timer.show_controls_popup = !timer.show_controls_popup;
                    }

                    // Removed Up/Down navigation since we no longer have a menu
                    KeyEvent {
                        code: KeyCode::Char('m'),
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => {
                        // Manual trigger for Mario animation (for testing)
                        timer.show_mario_animation = true;
                        timer.mario_animation = MarioAnimation::new();
                        timer.mario_animation.start();
                    }

                    _ => {}
                }
            }
        }

        // Update Mario animation
        if timer.show_mario_animation {
            timer.mario_animation.update();
            if timer.mario_animation.is_finished() {
                timer.show_mario_animation = false;
            }
        }

        // Check if timer finished
        if timer.current_session.is_running && timer.is_timer_finished() {
            timer.complete_session();
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run_timer() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(PomodoroTimer::format_duration(Duration::from_secs(0)), "00:00");
        assert_eq!(PomodoroTimer::format_duration(Duration::from_secs(30)), "00:30");
        assert_eq!(PomodoroTimer::format_duration(Duration::from_secs(60)), "01:00");
        assert_eq!(PomodoroTimer::format_duration(Duration::from_secs(125)), "02:05");
    }

    #[test]
    fn test_timer_creation() {
        let timer = PomodoroTimer::new().unwrap();
        assert_eq!(timer.mode, TimerMode::Auto);
        assert_eq!(timer.completed_sessions, 0);
        assert_eq!(timer.current_session.timer_type, TimerType::Work);
        assert!(!timer.current_session.is_running);
    }
}
