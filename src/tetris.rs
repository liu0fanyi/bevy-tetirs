// src/tetris.rs
use bevy::prelude::*;
use rand::Rng;
use std::time::Duration;

pub const FIELD_WIDTH: usize = 12;
pub const FIELD_HEIGHT: usize = 18;
pub const SCREEN_WIDTH: usize = 80; // Will likely be replaced by Bevy window config
pub const SCREEN_HEIGHT: usize = 30; // Will likely be replaced by Bevy window config

// Represents the 7 Tetromino shapes using a 4x4 grid.
// '.' means empty, 'X' means a block.
pub const TETROMINO_SHAPES: [&str; 7] = [
    "..X...X...X...X.", // I
    "..X..XX...X.....", // T
    ".....XX..XX.....", // O
    "..X..XX..X......", // L
    ".X...XX...X.....", // J
    ".X...X...XX.....", // S
    "..X...X..XX.....", // Z
];

// Function to rotate a point (px, py) in a 4x4 grid.
// r is the rotation state (0, 1, 2, 3).
pub fn rotate(px: i32, py: i32, r: i32) -> usize {
    let r_mod_4 = r % 4;
    let index = match r_mod_4 {
        0 => py * 4 + px,        // 0 degrees
        1 => 12 + py - (px * 4), // 90 degrees
        2 => 15 - (py * 4) - px, // 180 degrees
        3 => 3 - py + (px * 4),  // 270 degrees
        _ => unreachable!(),     // Should not happen due to modulo 4
    };
    index as usize
}

// Represents the game field.
// `Vec<u8>` stores the state of each cell.
// 0 means empty, other numbers might represent different Tetromino block types or colors.
// 9 could represent the border, as in the original C++ code.
#[derive(Resource)]
pub struct GameField {
    pub field: Vec<u8>,
}

impl GameField {
    pub fn new() -> Self {
        let mut field = vec![0; FIELD_WIDTH * FIELD_HEIGHT];
        // Initialize borders
        for y in 0..FIELD_HEIGHT {
            for x in 0..FIELD_WIDTH {
                if x == 0 || x == FIELD_WIDTH - 1 || y == FIELD_HEIGHT - 1 {
                    field[y * FIELD_WIDTH + x] = 9; // Border block
                }
            }
        }
        GameField { field }
    }

    // Helper to get a block at a certain coordinate
    pub fn get_block(&self, x: usize, y: usize) -> u8 {
        if x < FIELD_WIDTH && y < FIELD_HEIGHT {
            self.field[y * FIELD_WIDTH + x]
        } else {
            9 // Treat out of bounds as border for collision purposes
        }
    }

    // Helper to set a block at a certain coordinate
    pub fn set_block(&mut self, x: usize, y: usize, value: u8) {
        if x < FIELD_WIDTH && y < FIELD_HEIGHT {
            self.field[y * FIELD_WIDTH + x] = value;
        }
    }

    pub fn lock_piece(&mut self, piece: &CurrentPiece) {
        for py_local in 0..4 {
            for px_local in 0..4 {
                let piece_index = rotate(px_local, py_local, piece.rotation);
                if TETROMINO_SHAPES[piece.shape_index].chars().nth(piece_index) == Some('X') {
                    let field_x = piece.x + px_local;
                    let field_y = piece.y + py_local;

                    if field_x >= 0
                        && field_x < FIELD_WIDTH as i32
                        && field_y >= 0
                        && field_y < FIELD_HEIGHT as i32
                    {
                        // Add 1 because shape_index can be 0, and 0 is empty.
                        // Values 1-7 for pieces, 9 for border.
                        self.set_block(
                            field_x as usize,
                            field_y as usize,
                            (piece.shape_index + 1) as u8,
                        );
                    }
                }
            }
        }
    }

    // Returns the number of lines cleared
    pub fn check_and_clear_lines(&mut self) -> u32 {
        let mut actual_lines_cleared_this_call = 0;
        // Start checking from the bottom-most playable row.
        // FIELD_HEIGHT - 1 is the border.
        let mut write_row = FIELD_HEIGHT - 2;

        for read_row in (0..FIELD_HEIGHT - 1).rev() {
            // Iterate from bottom playable up to top
            let mut line_is_full = true;
            for x_check in 1..(FIELD_WIDTH - 1) {
                // Check within playable area (excluding side borders)
                if self.get_block(x_check, read_row) == 0 {
                    // If any cell is empty
                    line_is_full = false;
                    break;
                }
            }

            if line_is_full {
                actual_lines_cleared_this_call += 1;
                // Don't copy this line. `write_row` will not decrement.
                // Effectively, this line is "cleared" because it's skipped.
            } else {
                // This line is not full, so copy it to the `write_row` position
                // if `write_row` is different from `read_row` (i.e., lines below it were cleared)
                if write_row != read_row {
                    for x_copy in 1..(FIELD_WIDTH - 1) {
                        let block_to_copy = self.get_block(x_copy, read_row);
                        self.set_block(x_copy, write_row, block_to_copy);
                    }
                }
                // Ensure write_row doesn't go below 0 if FIELD_HEIGHT is very small or
                // if we are at the very top. The loop for read_row starts at 0,
                // so write_row must be protected if it's already 0 and we try to decrement.
                if write_row > 0 {
                    write_row -= 1; // Move to the next row upwards to write to.
                } else if write_row == 0 && read_row != 0 {
                    // This case means the top row was written to, but we are still reading lines from above.
                    // This shouldn't happen if read_row is always >= write_row after the first clear.
                    // If read_row is 0 here, it means the top line was not full, and it was copied to itself (if write_row was also 0).
                    // Then write_row would become -1, which is bad.
                    // Let's adjust: write_row should only decrement if it's above the effective top of the playfield.
                    // The playable rows are 0 to FIELD_HEIGHT - 2.
                    // write_row is an index.
                }
            }
        }

        // Fill the top rows (that were not written to by copying non-full lines) with empty blocks
        // `write_row` now indicates the highest row index that was written to by a non-full line,
        // or it's FIELD_HEIGHT - 2 if no lines were cleared.
        // If lines were cleared, write_row is now effectively the index of the highest non-cleared line
        // that was shifted down, or it has gone below zero if many lines were cleared.
        // The rows from 0 up to (and including) write_row need to be cleared if write_row is valid.
        // If actual_lines_cleared_this_call > 0, then rows from 0 to (actual_lines_cleared_this_call -1)
        // effectively become empty.
        // More simply: any row above the new position of the lowest non-cleared line becomes empty.
        // The variable `write_row` after the loop points to the row *just above* where the next non-full line would be written.
        // So, rows from 0 to `write_row` (inclusive) are the ones that need to be filled with 0s.
        // (adjusting write_row logic slightly to simplify this fill)

        // Corrected logic for filling top rows:
        // After the main loop, 'write_row' is the index of the row where the next non-full line from the *top*
        // would have been copied. So, all rows from 0 up to this 'write_row' (inclusive, if it's valid)
        // are now empty because their content was shifted down or they were part of cleared lines.
        for y_fill_top in 0..=write_row {
            if y_fill_top >= FIELD_HEIGHT - 1 {
                continue;
            } // Should not happen if write_row logic is correct
            for x_fill_top in 1..(FIELD_WIDTH - 1) {
                self.set_block(x_fill_top, y_fill_top, 0);
            }
        }

        if actual_lines_cleared_this_call > 0 {
            println!(
                "Internal: Lines cleared this call: {}",
                actual_lines_cleared_this_call
            );
        }
        actual_lines_cleared_this_call
    }
}

#[derive(Resource)]
pub struct CurrentPiece {
    pub shape_index: usize, // Index into TETROMINO_SHAPES
    pub rotation: i32,
    pub x: i32, // Position on the game field (top-left of the 4x4 grid)
    pub y: i32,
}

impl CurrentPiece {
    pub fn new(shape_index: usize) -> Self {
        CurrentPiece {
            shape_index,
            rotation: 0,
            x: (FIELD_WIDTH / 2) as i32 - 2, // Start roughly in the middle
            y: 0,                            // Start at the top
        }
    }
}

#[derive(Resource, Default)]
pub struct Score(pub u32);

#[derive(Resource)]
pub struct GameTimer {
    pub fall_timer: Timer, // Timer that dictates when a piece should attempt to fall
    pub current_fall_interval_seconds: f32,
    // speed_level can be a separate resource or integrated if difficulty changes often
}

impl GameTimer {
    pub fn new(initial_speed_level: u32) -> Self {
        // initial_speed_level = 20 means 20 * 50ms = 1.0 second interval
        let fall_interval_seconds = initial_speed_level as f32 * 0.05;
        GameTimer {
            fall_timer: Timer::from_seconds(fall_interval_seconds, TimerMode::Repeating),
            current_fall_interval_seconds: fall_interval_seconds,
        }
    }

    // Optional: Method to change speed later
    pub fn set_fall_interval(&mut self, seconds: f32) {
        self.current_fall_interval_seconds = seconds;
        self.fall_timer
            .set_duration(Duration::from_secs_f32(seconds));
        self.fall_timer.reset();
    }
}

// GameSpeed is essentially managed by GameTimer.speed_level and piece_count for now.
// We can add a separate GameSpeed resource if more complex logic is needed later.

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    Playing,
    GameOver,
}

// ... (ensure TETROMINO_SHAPES, rotate, FIELD_WIDTH, FIELD_HEIGHT, GameField are in scope) ...

pub fn does_piece_fit(
    field: &GameField,
    shape_index: usize,
    rotation: i32,
    pos_x: i32, // Target X position of the piece's 4x4 grid top-left
    pos_y: i32, // Target Y position of the piece's 4x4 grid top-left
) -> bool {
    for py_local in 0..4 {
        // py_local is py within the 4x4 piece grid
        for px_local in 0..4 {
            // px_local is px within the 4x4 piece grid
            let piece_index = rotate(px_local, py_local, rotation);

            if TETROMINO_SHAPES[shape_index].chars().nth(piece_index) == Some('X') {
                // This cell in the piece is a block. Check its position on the field.
                let field_x = pos_x + px_local;
                let field_y = pos_y + py_local;

                // If an 'X' block is trying to go out of the defined playfield boundaries, it's a fail.
                if field_x < 0
                    || field_x >= FIELD_WIDTH as i32
                    || field_y < 0
                    || field_y >= FIELD_HEIGHT as i32
                {
                    return false; // Piece block is out of bounds
                }

                // Current cell is within field bounds. Check for collision with existing blocks.
                // Note: Borders (value 9) are also considered occupied.
                if field.get_block(field_x as usize, field_y as usize) != 0 {
                    return false; // Collision with an existing block or border
                }
            }
        }
    }
    true // No collisions found, piece fits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotate_0_degrees() {
        // Example: point (1,0) in a 4x4 grid
        // . X . .
        // . . . .
        // . . . .
        // . . . .
        // Expected index: 0*4 + 1 = 1
        assert_eq!(rotate(1, 0, 0), 1);
    }

    #[test]
    fn test_rotate_90_degrees() {
        // Example: point (1,0) rotated 90 degrees
        // Becomes (0,2) effectively in the new orientation from top-left.
        // Original:
        // . X . . (px=1, py=0) -> index 1
        // Rotated 90 deg (visual representation):
        // . . . .
        // . . . .
        // X . . . (px=0, py=2 in the *new* orientation of the piece)
        // Formula: 12 + py - (px * 4)
        // For (px=1, py=0) from original: 12 + 0 - (1 * 4) = 8
        assert_eq!(rotate(1, 0, 1), 8);
    }

    #[test]
    fn test_rotate_180_degrees() {
        // Example: point (1,0) rotated 180 degrees
        // Formula: 15 - (py * 4) - px
        // For (px=1, py=0): 15 - (0*4) - 1 = 14
        assert_eq!(rotate(1, 0, 2), 14);
    }

    #[test]
    fn test_rotate_270_degrees() {
        // Example: point (1,0) rotated 270 degrees
        // Formula: 3 - py + (px * 4)
        // For (px=1, py=0): 3 - 0 + (1*4) = 7
        assert_eq!(rotate(1, 0, 3), 7);
    }

    #[test]
    fn test_game_field_init() {
        let game_field = GameField::new();
        // Check a border cell
        assert_eq!(game_field.get_block(0, 0), 9);
        // Check an inner cell
        assert_eq!(game_field.get_block(1, 1), 0);
        // Check bottom border
        assert_eq!(game_field.get_block(5, FIELD_HEIGHT - 1), 9);
    }

    #[test]
    fn test_does_piece_fit_empty_field_clear_center() {
        let field = GameField::new(); // Borders are set, middle is empty
                                      // Try to place 'I' tetromino (index 0) at y=0, x should allow it if centered
                                      // I-shape: "..X...X...X...X." (Xs at local x=2 for y=0,1,2,3)
                                      // Centering: FIELD_WIDTH / 2 - 2 (for the 4x4 grid)
        let pos_x = (FIELD_WIDTH / 2) as i32 - 2;
        assert!(
            does_piece_fit(&field, 0, 0, pos_x, 0),
            "I-shape should fit in empty field center"
        );
    }

    #[test]
    fn test_does_piece_fit_collision_with_left_border() {
        let field = GameField::new();
        // I-shape (index 0) has its first 'X' at local px_local=2.
        // If piece pos_x = -2, then field_x for this block = -2 + 2 = 0, which is a border.
        assert!(
            !does_piece_fit(&field, 0, 0, -2, 0),
            "Should collide with left border (block at field x=0)"
        );
    }

    #[test]
    fn test_does_piece_fit_out_of_bounds_left() {
        let field = GameField::new();
        // I-shape (index 0), block at px_local=2.
        // If piece pos_x = -3, then field_x for this block = -3 + 2 = -1, which is out of bounds.
        assert!(
            !does_piece_fit(&field, 0, 0, -3, 0),
            "Should be false if 'X' block is out of bounds left"
        );
    }

    #[test]
    fn test_does_piece_fit_collision_with_bottom_border() {
        let field = GameField::new();
        // I-shape (index 0) has its last 'X' at local py_local=3 (and px_local=2).
        // If piece pos_y = FIELD_HEIGHT as i32 - 4, this block's field_y = (FIELD_HEIGHT-4)+3 = FIELD_HEIGHT-1 (border).
        assert!(
            !does_piece_fit(&field, 0, 0, 5, (FIELD_HEIGHT - 4) as i32),
            "Should collide with bottom border"
        );
    }

    #[test]
    fn test_does_piece_fit_out_of_bounds_bottom() {
        let field = GameField::new();
        // I-shape (index 0), block at py_local=3.
        // If piece pos_y = FIELD_HEIGHT as i32 - 3, this block's field_y = (FIELD_HEIGHT-3)+3 = FIELD_HEIGHT (out of bounds).
        assert!(
            !does_piece_fit(&field, 0, 0, 5, (FIELD_HEIGHT - 3) as i32),
            "Should be false if 'X' block is out of bounds bottom"
        );
    }

    #[test]
    fn test_does_piece_fit_collision_with_existing_block() {
        let mut field = GameField::new();
        field.set_block(5, 2, 1); // Place an existing block (value 1)
                                  // 'I' tetromino (index 0) has a block at its local (px_local=2, py_local=1).
                                  // If piece is at pos_x=3, pos_y=1, its block at (2,1) will target field coordinates (3+2, 1+1) = (5,2).
        assert!(
            !does_piece_fit(&field, 0, 0, 3, 1),
            "Should collide with existing block at (5,2)"
        );
    }

    #[test]
    fn test_does_piece_fit_o_shape_near_border() {
        // O-shape: ".....XX..XX....." (local x=1,y=1; x=2,y=1; x=1,y=2; x=2,y=2)
        let field = GameField::new();
        // Place O-shape (index 2) at (0,0). Its leftmost blocks (px_local=1) will be at field_x=1. Valid.
        assert!(
            does_piece_fit(&field, 2, 0, 0, 0),
            "O-shape at (0,0) should fit if its blocks are not on border index 0"
        );
        // Place O-shape at (-1,0). Its leftmost blocks (px_local=1) will be at field_x=0 (border). Collision.
        assert!(
            !does_piece_fit(&field, 2, 0, -1, 0),
            "O-shape at (-1,0) should collide with left border"
        );
    }
}
