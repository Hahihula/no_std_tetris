use std::io::{self, Stdout, Write};
use std::sync::mpsc::{Receiver, channel};
use std::thread;
use std::time::{Duration, Instant};
use tetris_game::{Tetris, draw_on_screen};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, ClearType},
};
use ctrlc;

struct TerminalWriter {
    stdout: Stdout,
}

impl TerminalWriter {
    fn new() -> Self {
        TerminalWriter {
            stdout: io::stdout(),
        }
    }
}

impl core::fmt::Write for TerminalWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        write!(self.stdout, "{}", s).map_err(|_| core::fmt::Error)?;
        self.stdout.flush().map_err(|_| core::fmt::Error)?;
        Ok(())
    }
}

fn main() -> io::Result<()> {
    // Initialize terminal
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?;

    // Signal handling
    let (signal_tx, signal_rx) = channel();
    ctrlc::set_handler(move || {
        let _ = signal_tx.send(true);
    })
    .expect("Error setting Ctrl+C handler");

    let mut game = Tetris::new();
    let mut writer = TerminalWriter::new();
    let mut last_update = Instant::now();
    let fall_interval = Duration::from_millis(500);

    // Debouncing
    let mut last_key_time = Instant::now();
    let debounce_duration = Duration::from_millis(100); // 100ms debounce

    // Game loop
    'game_loop: loop {
        // Check for Ctrl+C signal
        if let Ok(true) = signal_rx.try_recv() {
            break 'game_loop;
        }

        // Handle timing
        let now = Instant::now();
        if now - last_update >= fall_interval {
            game.move_down();
            last_update = now;
        }

        // Handle input with debouncing
        if event::poll(Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()? {
                let time_since_last_key = now - last_key_time;
                if time_since_last_key >= debounce_duration {
                    match key {
                        KeyEvent {
                            code: KeyCode::Esc, ..
                        } => break 'game_loop,
                        KeyEvent {
                            code: KeyCode::Char('q'),
                            ..
                        } => break 'game_loop,
                        KeyEvent {
                            code: KeyCode::Left,
                            ..
                        } => {
                            game.move_left();
                        }
                        KeyEvent {
                            code: KeyCode::Right,
                            ..
                        } => {
                            game.move_right();
                        }
                        KeyEvent {
                            code: KeyCode::Down,
                            ..
                        } => {
                            game.move_down();
                        }
                        KeyEvent {
                            code: KeyCode::Up, ..
                        } => {
                            game.rotate();
                        }
                        KeyEvent {
                            code: KeyCode::Char('c'),
                            modifiers: KeyModifiers::CONTROL,
                            ..
                        } => {
                            break 'game_loop;
                        }
                        _ => {}
                    }
                    last_key_time = now;
                }
            }
        }

        // Draw game
        execute!(writer.stdout, terminal::Clear(ClearType::All))?;
        match draw_on_screen(&game, &mut writer) {
            Ok(_) => {}
            Err(e) => {
                println!("Error drawing game: {}", e);
            }
        };

        if game.is_game_over() {
            thread::sleep(Duration::from_secs(2));
            break 'game_loop;
        }

        thread::sleep(Duration::from_millis(16));
    }

    // Cleanup
    execute!(stdout, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    println!("Thanks for playing! You scored {} points.", game.score);
    Ok(())
}
