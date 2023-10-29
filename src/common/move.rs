use std::fmt;

use super::{board::Board, piece::Piece, square::Square};

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

    pub fn as_notation(&self, board: &Board) -> String {
        let Move { from, to } = self;
        let piece = board.piece_on(from).unwrap();
        let mut move_text = String::from("");
        if piece == Piece::King {
            if (from, to) == (&Square::E1, &Square::G1) || (from, to) == (&Square::E8, &Square::G8)
            {
                return String::from("0-0");
            } else if (from, to) == (&Square::E1, &Square::C1)
                || (from, to) == (&Square::E8, &Square::C8)
            {
                return String::from("0-0-0");
            }
        } else {
            let is_capture = board.piece_on(to).is_some();
            match piece {
                Piece::Pawn => {
                    move_text = if is_capture {
                        format!("{}x{}", from, to)
                    } else {
                        format!("{}", to)
                    }
                }
                _ => {
                    move_text = if is_capture {
                        format!("{}x{}", piece, to)
                    } else {
                        format!("{}{}", piece, to)
                    }
                }
            }
        }
        dbg!(&move_text);
        move_text
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.from, self.to)
    }
}
