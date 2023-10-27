use std::fmt;

use super::color::Color;

/// Represent a chess piece.
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Debug)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

/// Numbers of [`Piece`].
pub const NUM_PIECES: usize = 6;

/// An array representing each [`Piece`] type, in order of ascending value.
pub const ALL_PIECES: [Piece; NUM_PIECES] = [
    Piece::Pawn,
    Piece::Knight,
    Piece::Bishop,
    Piece::Rook,
    Piece::Queen,
    Piece::King,
];

// /// Number of promotion.
// pub const NUM_PROMOTION_PIECES: usize = 4;

// /// Enumerate all [`Piece`] in which a [`Piece::Pawn`] can be promoted.
// pub const PROMOTION_PIECES: [Piece; 4] = [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight];

impl Piece {
    /// Convert the [`Piece`] to a [`usize`].
    #[inline]
    pub fn to_index(&self) -> usize {
        *self as usize
    }

    /// Convert a piece with a [`Color`] to a string.
    ///
    /// > **Note**: White pieces are uppercase, black pieces are lowercase.
    ///
    /// ```
    /// use chess::{Piece, Color};
    ///
    /// assert_eq!(Piece::King.to_fen_string(Color::White), "K");
    /// assert_eq!(Piece::Knight.to_fen_string(Color::Black), "n");
    /// ```
    #[inline]
    pub fn to_fen_string(&self, color: Color) -> String {
        let piece = format!("{}", self);
        match color {
            Color::White => piece,
            Color::Black => piece.to_lowercase(),
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Piece::Pawn => "P",
                Piece::Knight => "N",
                Piece::Bishop => "B",
                Piece::Rook => "R",
                Piece::Queen => "Q",
                Piece::King => "K",
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_index() {
        assert_eq!(Piece::Pawn.to_index(), 0);
        assert_eq!(Piece::Knight.to_index(), 1);
        assert_eq!(Piece::Bishop.to_index(), 2);
        assert_eq!(Piece::Rook.to_index(), 3);
        assert_eq!(Piece::Queen.to_index(), 4);
        assert_eq!(Piece::King.to_index(), 5);
    }

    #[test]
    fn to_string_per_color() {
        assert_eq!(Piece::Pawn.to_fen_string(Color::White), "P");
        assert_eq!(Piece::Knight.to_fen_string(Color::White), "N");
        assert_eq!(Piece::Bishop.to_fen_string(Color::White), "B");
        assert_eq!(Piece::Rook.to_fen_string(Color::White), "R");
        assert_eq!(Piece::Queen.to_fen_string(Color::White), "Q");
        assert_eq!(Piece::King.to_fen_string(Color::White), "K");

        assert_eq!(Piece::Pawn.to_fen_string(Color::Black), "p");
        assert_eq!(Piece::Knight.to_fen_string(Color::Black), "n");
        assert_eq!(Piece::Bishop.to_fen_string(Color::Black), "b");
        assert_eq!(Piece::Rook.to_fen_string(Color::Black), "r");
        assert_eq!(Piece::Queen.to_fen_string(Color::Black), "q");
        assert_eq!(Piece::King.to_fen_string(Color::Black), "k");
    }

    #[test]
    fn fmt() {
        assert_eq!(format!("{}", Piece::Pawn), "P".to_string());
        assert_eq!(format!("{}", Piece::Knight), "N".to_string());
        assert_eq!(format!("{}", Piece::Bishop), "B".to_string());
        assert_eq!(format!("{}", Piece::Rook), "R".to_string());
        assert_eq!(format!("{}", Piece::Queen), "Q".to_string());
        assert_eq!(format!("{}", Piece::King), "K".to_string());
    }
}
