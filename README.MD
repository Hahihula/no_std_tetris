# no_std_tetris

A minimalist, flexible Tetris implementation in Rust that works in both `no_std` and standard environments. This crate provides a core Tetris game engine that can be used on embedded systems, in terminal applications, or in any Rust project needing Tetris gameplay.

[![Crates.io](https://img.shields.io/crates/v/no_std_tetris.svg)](https://crates.io/crates/no_std_tetris)
[![Documentation](https://docs.rs/no_std_tetris/badge.svg)](https://docs.rs/no_std_tetris)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

## Project structure
```
no_std_tetris/
├── no_std_tetris           # the crate
├── terminal_example        # terminal example
├── esp32_c3_example        # example implementation on esp32-c3
├── LICENSE
└── README.md
```