use std::ops::Not;

use serde::Serialize;

/// Represent a color in Chess game.
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Debug, Serialize)]
pub enum Color {
    White,
    Black,
}

impl Color {
    /// Convert the [`Color`] to a [`usize`] for table lookups.
    #[inline]
    pub fn as_index(&self) -> usize {
        *self as usize
    }
}

impl Not for Color {
    type Output = Self;

    /// Get the other color.
    #[inline]
    fn not(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}
