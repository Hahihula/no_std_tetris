# no_std_tetris

A minimalist, flexible Tetris implementation in Rust that works in both `no_std` and standard environments. This crate provides a core Tetris game engine that can be used on embedded systems, in terminal applications, or in any Rust project needing Tetris gameplay.

[![Crates.io](https://img.shields.io/crates/v/no_std_tetris.svg)](https://crates.io/crates/no_std_tetris)
[![Documentation](https://docs.rs/no_std_tetris/badge.svg)](https://docs.rs/no_std_tetris)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

## Features

- Works in `no_std` environments, perfect for embedded systems
- Generic over random number generation, allowing for platform-specific implementations
- Simple API for game control (move, rotate, etc.)
- Customizable rendering through a generic drawing function
- Complete terminal example using crossterm

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
no_std_tetris = "0.1.0"
```

## API Overview

### Core Types
  * `Tetris<R>`: The main game state, generic over a random number generator
  * `RandomGenerator`: Trait for providing random numbers to select new pieces
  * `Color`: Enum for piece colors
  * `Tetromino`: Struct representing a tetromino pieces

### Game Controls
```rust
// Create a new game with a random number generator
let mut game = Tetris::new(rng);

// Game controls
game.move_left();
game.move_right();
game.move_down();
game.rotate();

// Check game status
if game.is_game_over() {
    println!("Game Over! Score: {}", game.score);
}
```

### Rendering
The library provides a default text-based renderer, but you usually will implement your own. The example implementation is:
```rust
pub fn draw_on_screen<R: RandomGenerator>(
    tetris: &Tetris<R>,
    f: &mut impl fmt::Write,
) -> fmt::Result {
    for y in 0..BOARD_HEIGHT {
        write!(f, "|")?;
        for x in 0..BOARD_WIDTH {
            let mut occupied = tetris.board[y][x].is_some();
            if !tetris.game_over {
                for &(dx, dy) in &tetris.current_piece.shape {
                    if (tetris.piece_pos.0 + dx as i8) as usize == x
                        && (tetris.piece_pos.1 + dy as i8) as usize == y
                    {
                        occupied = true;
                    }
                }
            }
            write!(f, "{}", if occupied { "#" } else { " " })?;
        }
        writeln!(f, "|")?;
    }
    if tetris.game_over {
        writeln!(f, "GAME OVER - Score: {}", tetris.score)
    } else {
        writeln!(f, "Score: {}", tetris.score)
    }
}
```