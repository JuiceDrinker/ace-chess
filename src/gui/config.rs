//! Config file that defines every constant
//!
//! | Component | axes   |
//! |-----------|--------|
//! | foo.0     | x-axes |
//! | foo.1     | y-axes |

use crate::prelude::BOARD_SIZE;

/// The pixel-size of the board only (the board have a square form).
pub const BOARD_PX_SIZE: (f32, f32) = (800.0, 800.0);

/// The pixel-size of the side screen.
pub const SIDE_SCREEN_PX_SIZE: (f32, f32) = (360.0, BOARD_PX_SIZE.1);

/// The pixel-size of the screen.
pub const SCREEN_PX_SIZE: (f32, f32) = (BOARD_PX_SIZE.0 + SIDE_SCREEN_PX_SIZE.0, BOARD_PX_SIZE.1);

/// The pixel-size of a Board's cell.
pub const BOARD_CELL_PX_SIZE: (f32, f32) = (
    BOARD_PX_SIZE.0 / BOARD_SIZE.0 as f32,
    BOARD_PX_SIZE.1 / BOARD_SIZE.1 as f32,
);
