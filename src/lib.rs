#![no_std]

use core::fmt;

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

// Tetromino shapes with their rotations
const TETROMINOS: &[([(u8, u8); 4], Color); 7] = &[
    ([(0, 1), (1, 1), (2, 1), (3, 1)], Color::Cyan), // I
    ([(0, 0), (0, 1), (1, 0), (1, 1)], Color::Yellow), // O
    ([(0, 1), (1, 1), (2, 1), (1, 0)], Color::Magenta), // T
    ([(0, 0), (1, 0), (2, 0), (2, 1)], Color::Green), // L
    ([(0, 1), (1, 1), (2, 1), (2, 0)], Color::Red),  // J
    ([(0, 0), (1, 0), (1, 1), (2, 1)], Color::Blue), // S
    ([(0, 1), (1, 1), (1, 0), (2, 0)], Color::White), // Z
];

// Game state
pub struct Tetris {
    board: [[Option<Color>; 10]; 20],
    current_piece: [(u8, u8); 4],
    piece_pos: (i8, i8),
    piece_color: Color,
    pub score: u32,
    game_over: bool,
}

impl Tetris {
    pub fn new() -> Self {
        let mut game = Tetris {
            board: [[None; 10]; 20],
            current_piece: TETROMINOS[0].0,
            piece_pos: (3, 0),
            piece_color: TETROMINOS[0].1,
            score: 0,
            game_over: false,
        };
        // Check initial spawn
        if !game.can_place(&game.current_piece, game.piece_pos) {
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
            rotated[i] = (self.current_piece[i].1, 3 - self.current_piece[i].0);
        }

        if self.can_place(&rotated, self.piece_pos) {
            self.current_piece = rotated;
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
        if self.can_place(&self.current_piece, new_pos) {
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
                || x >= 10
                || y >= 20
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
        for &(dx, dy) in &self.current_piece {
            let x = (self.piece_pos.0 + dx as i8) as usize;
            let y = (self.piece_pos.1 + dy as i8) as usize;
            self.board[y][x] = Some(self.piece_color);
        }
        self.check_lines();
    }

    fn spawn_new_piece(&mut self) {
        if self.game_over {
            return;
        }
        // Simple random selection (in real impl would need RNG)
        let idx = (self.score % 7) as usize;
        self.current_piece = TETROMINOS[idx].0;
        self.piece_color = TETROMINOS[idx].1;
        self.piece_pos = (3, 0);

        // Check if new piece can be placed, if not, game over
        if !self.can_place(&self.current_piece, self.piece_pos) {
            self.game_over = true;
        }
    }

    fn check_lines(&mut self) {
        if self.game_over {
            return;
        }
        for y in 0..20 {
            if self.board[y].iter().all(|&cell| cell.is_some()) {
                // Clear line
                for yy in (1..=y).rev() {
                    self.board[yy] = self.board[yy - 1];
                }
                self.board[0] = [None; 10];
                self.score += 100;
            }
        }
    }
}

// Separate drawing function
pub fn draw_on_screen(tetris: &Tetris, f: &mut impl fmt::Write) -> fmt::Result {
    for y in 0..20 {
        write!(f, "|")?;
        for x in 0..10 {
            let mut occupied = tetris.board[y][x].is_some();
            if !tetris.game_over {
                for &(dx, dy) in &tetris.current_piece {
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
