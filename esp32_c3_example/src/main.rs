//! Runs TETRIS on display from sk6812 RGBW LED strip and buttons using interrupts.
//!
//! The following wiring is assumed:
//! - LED => GPIO8
//! - RIGHT_BUTTON => GPIO0 -> GND
//! - MIDDLE_BUTTON => GPIO1 -> GND
//! - LEFT_BUTTON => GPIO2 -> GND
//! - LED_STRIP_DATA => GPIO4
//!

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    gpio::{Input, Level, Output, Pull},
    rmt::{PulseCode, TxChannelConfig, TxChannelCreator},
    time::Rate,
};
use esp_println::println;
use no_std_tetris::{RandomGenerator, Tetris, Color};

// global config
const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 20;
const FALL_INTERVAL_MS: u64 = 500;
const BRIGHTNESS: u8 = 12;
// // neopixel
const T0H: u16 = 35; // 437.5 ns
const T0L: u16 = 90; // 1125 ns
const T1H: u16 = 70; // 875 ns
const T1L: u16 = 55; // 687.5 ns

struct TetrisRng;

impl RandomGenerator for TetrisRng {
    fn next_random(&mut self) -> usize {
        // Simple LFSR-like pseudo-random for embedded
        static mut STATE: u32 = 0x12345678;
        unsafe {
            let state = STATE;
            let new_state = state.wrapping_mul(1103515245).wrapping_add(12345);
            STATE = new_state;
            (new_state >> 16) as usize
        }
    }
}

// neopixel LED strip config
fn create_led_bits(r: u8, g: u8, b: u8) -> [PulseCode; 25] {
    let mut data = [PulseCode::default(); 25];

    // WS2812B expects GRB order
    let bytes = [g, r, b];

    let mut idx = 0;
    for byte in bytes {
        for bit in (0..8).rev() {
            data[idx] = if (byte & (1 << bit)) != 0 {
                PulseCode::new(Level::High, T1H, Level::Low, T1L)
            } else {
                PulseCode::new(Level::High, T0H, Level::Low, T0L)
            };
            idx += 1;
        }
    }
    data[24] = PulseCode::new(Level::Low, 800, Level::Low, 0); // Reset code
    data
}

fn color_to_rgb(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Red => (BRIGHTNESS, 0, 0),
        Color::Green => (0, BRIGHTNESS, 0),
        Color::Blue => (0, 0, BRIGHTNESS),
        Color::Yellow => (BRIGHTNESS / 2, BRIGHTNESS / 2, 0),
        Color::Cyan => (0, BRIGHTNESS / 2, BRIGHTNESS / 2),
        Color::Magenta => (BRIGHTNESS / 2, 0, BRIGHTNESS / 2),
        Color::White => (BRIGHTNESS / 3, BRIGHTNESS / 3, BRIGHTNESS / 3),
        _ => (0, 0, 0),
    }
}

fn board_to_led_index(x: usize, y: usize, flip_y: bool) -> usize {
    let y_mapped = if flip_y { BOARD_HEIGHT - 1 - y } else { y };
    let col_start = x * BOARD_HEIGHT; // Each column has 20 LEDs
    if x % 2 == 0 {
        // Even columns: y=0 at top (or bottom if flipped), y=19 at bottom (or top)
        col_start + y_mapped
    } else {
        // Odd columns: y=0 at bottom (or top if flipped), y=19 at top (or bottom)
        col_start + (BOARD_HEIGHT - 1 - y_mapped)
    }
}

#[esp_rtos::main]
async fn main(_spawner: Spawner) {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Status LED on GPIO8
    let mut status_led = Output::new(peripherals.GPIO8, Level::Low, Default::default());

    // Button inputs with pull-ups
    let in_config = esp_hal::gpio::InputConfig::default().with_pull(Pull::Up);
    let right_button = Input::new(peripherals.GPIO0, in_config.clone());
    let middle_button = Input::new(peripherals.GPIO1, in_config.clone());
    let left_button = Input::new(peripherals.GPIO2, in_config);

    // RMT setup for NeoPixel on GPIO4
    let rmt = esp_hal::rmt::Rmt::new(peripherals.RMT, Rate::from_mhz(80))
        .unwrap()
        .into_async();

    let tx_config = TxChannelConfig::default()
        .with_clk_divider(1)
        .with_idle_output_level(Level::Low)
        .with_idle_output(false);

    let mut channel = rmt
        .channel0
        .configure_tx(&tx_config)
        .unwrap()
        .with_pin(peripherals.GPIO4);

    let mut game = Tetris::new(TetrisRng);
    let mut last_update = embassy_time::Instant::now();
    let fall_interval = embassy_time::Duration::from_millis(FALL_INTERVAL_MS);

    let mut last_key_time = embassy_time::Instant::now();
    let debounce_duration = embassy_time::Duration::from_millis(250);

    println!("Tetris game started!");

    // Game loop
    loop {
        let now = embassy_time::Instant::now();

        // Handle auto-fall timing
        if now - last_update >= fall_interval {
            game.move_down();
            last_update = now;
        }

        // Handle button inputs with debounce
        if right_button.is_low() {
            if now - last_key_time > debounce_duration {
                last_key_time = now;
                game.move_right();
            }
        }
        if left_button.is_low() {
            if now - last_key_time > debounce_duration {
                last_key_time = now;
                game.move_left();
            }
        }
        if middle_button.is_low() {
            if now - last_key_time > debounce_duration {
                last_key_time = now;
                game.rotate();
            }
        }

        let mut led_colors = [(0u8, 0u8, 0u8); 200];

        // Render board state
        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                if let Some(color) = game.board[y][x] {
                    let led_idx = board_to_led_index(x, y, true);
                    led_colors[led_idx] = color_to_rgb(color);
                }
            }
        }

        // Render current piece (if not game over)
        if !game.is_game_over() {
            for &(dx, dy) in &game.current_piece.shape {
                let x = (game.piece_pos.0 + dx as i8) as usize;
                let y = (game.piece_pos.1 + dy as i8) as usize;
                if x < BOARD_WIDTH && y < BOARD_HEIGHT {
                    let led_idx = board_to_led_index(x, y, true);
                    led_colors[led_idx] = color_to_rgb(game.current_piece.color);
                }
            }
        }

        // Send data to LED strip
        for (_, &(r, g, b)) in led_colors.iter().enumerate() {
            let data = create_led_bits(r, g, b);
            match channel.transmit(&data).await {
                Ok(_) => {}
                Err(err) => {
                    println!("Error transmitting LED data: {:?}", err);
                    break;
                }
            }
        }

        // Small delay to prevent busy-waiting
        embassy_time::Timer::after_millis(10).await;
    }
}