#![no_std]

use core::fmt;

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 20;

// Define colors
#[derive(Clone, Copy)]
pub enum Color {
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
    White,
}

// Tetromino struct
#[derive(Clone, Copy)]
pub struct Tetromino {
    pub shape: [(u8, u8); 4],
    pub color: Color,
}

// Tetromino shapes with their rotations
const TETROMINOS: &[Tetromino; 7] = &[
    Tetromino {
        shape: [(0, 1), (1, 1), (2, 1), (3, 1)],
        color: Color::Cyan,
    }, // I
    Tetromino {
        shape: [(0, 0), (0, 1), (1, 0), (1, 1)],
        color: Color::Yellow,
    }, // O
    Tetromino {
        shape: [(0, 1), (1, 1), (2, 1), (1, 0)],
        color: Color::Magenta,
    }, // T
    Tetromino {
        shape: [(0, 0), (1, 0), (2, 0), (2, 1)],
        color: Color::Green,
    }, // L
    Tetromino {
        shape: [(0, 1), (1, 1), (2, 1), (2, 0)],
        color: Color::Red,
    }, // J
    Tetromino {
        shape: [(0, 0), (1, 0), (1, 1), (2, 1)],
        color: Color::Blue,
    }, // S
    Tetromino {
        shape: [(0, 1), (1, 1), (1, 0), (2, 0)],
        color: Color::White,
    }, // Z
];

// Game state
pub struct Tetris<R: RandomGenerator> {
    pub board: [[Option<Color>; BOARD_WIDTH]; BOARD_HEIGHT],
    pub current_piece: Tetromino,
    pub piece_pos: (i8, i8),
    pub score: u32,
    pub game_over: bool,
    rng: R,
}

pub trait RandomGenerator {
    fn next_random(&mut self) -> usize;
}

impl<R: RandomGenerator> Tetris<R> {
    pub fn new(rng: R) -> Self {
        let mut game = Tetris {
            board: [[None; BOARD_WIDTH]; BOARD_HEIGHT],
            current_piece: TETROMINOS[0].clone(),
            piece_pos: (3, 0),
            score: 0,
            game_over: false,
            rng,
        };
        // Check initial spawn
        if !game.can_place(&game.current_piece.shape, game.piece_pos) {
            game.game_over = true;
        }
        game
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    // Control functions
    pub fn move_left(&mut self) -> bool {
        self.try_move((-1, 0))
    }

    pub fn move_right(&mut self) -> bool {
        self.try_move((1, 0))
    }

    pub fn move_down(&mut self) -> bool {
        if self.game_over {
            return false;
        }
        if !self.try_move((0, 1)) {
            self.lock_piece();
            self.spawn_new_piece();
            true
        } else {
            false
        }
    }

    pub fn rotate(&mut self) -> bool {
        if self.game_over {
            return false;
        }
        let mut rotated = [(0, 0); 4];
        for i in 0..4 {
            rotated[i] = (
                self.current_piece.shape[i].1,
                3 - self.current_piece.shape[i].0,
            );
        }

        if self.can_place(&rotated, self.piece_pos) {
            self.current_piece.shape = rotated;
            true
        } else {
            false
        }
    }

    fn try_move(&mut self, delta: (i8, i8)) -> bool {
        if self.game_over {
            return false;
        }
        let new_pos = (self.piece_pos.0 + delta.0, self.piece_pos.1 + delta.1);
        if self.can_place(&self.current_piece.shape, new_pos) {
            self.piece_pos = new_pos;
            true
        } else {
            false
        }
    }

    fn can_place(&self, piece: &[(u8, u8); 4], pos: (i8, i8)) -> bool {
        for &(dx, dy) in piece {
            let x = pos.0 + dx as i8;
            let y = pos.1 + dy as i8;
            if x < 0
                || x >= BOARD_WIDTH as i8
                || y >= BOARD_HEIGHT as i8
                || (y >= 0 && self.board[y as usize][x as usize].is_some())
            {
                return false;
            }
        }
        true
    }

    fn lock_piece(&mut self) {
        if self.game_over {
            return;
        }
        for &(dx, dy) in &self.current_piece.shape {
            let x = (self.piece_pos.0 + dx as i8) as usize;
            let y = (self.piece_pos.1 + dy as i8) as usize;
            self.board[y][x] = Some(self.current_piece.color);
        }
        self.check_lines();
    }

    // fn select_new_piece(tetrominos) {

    fn spawn_new_piece(&mut self) {
        if self.game_over {
            return;
        }
        // Simple random selection (in real impl would need RNG)
        let idx = self.rng.next_random() % 7;
        self.current_piece = TETROMINOS[idx].clone();
        self.piece_pos = (3, 0);

        // Check if new piece can be placed, if not, game over
        if !self.can_place(&self.current_piece.shape, self.piece_pos) {
            self.game_over = true;
        }
    }

    fn check_lines(&mut self) {
        if self.game_over {
            return;
        }
        for y in 0..BOARD_HEIGHT {
            if self.board[y].iter().all(|&cell| cell.is_some()) {
                // Clear line
                for yy in (1..=y).rev() {
                    self.board[yy] = self.board[yy - 1];
                }
                self.board[0] = [None; BOARD_WIDTH];
                self.score += 100;
            }
        }
    }
}

// Example drawing function
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
