use std::fmt;

use super::square::Square;

/// Represent a Move
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Move {
    /// The [`Square`] where the [`Piece`] comes from.
    pub from: Square,
    /// The [`Square`] where the [`Piece`] is going to.
    pub to: Square,
}

impl Move {
    /// Create a new chess move.
    #[inline]
    pub fn new(from: Square, to: Square) -> Self {
        Move { from, to }
    }

    /// The distance between the two [`Square`] of the move.
    ///
    /// ```
    /// use chess::{Move, Square};
    ///
    /// let m = Move::new(Square::A1, Square::H8);
    ///
    /// assert_eq!(m.distance(), 7);
    /// ```
    pub fn distance(&self) -> u32 {
        self.from.distance(self.to)
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.from, self.to)
    }
}
