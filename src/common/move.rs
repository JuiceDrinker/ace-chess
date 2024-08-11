use std::fmt;

use crate::{
    common::{color::Color, rank::Rank},
    error::{Error, ParseKind},
    logic::movetree::treenode::{CMove, CMoveKind, CastleSide, MoveDetails},
};

use super::{board::Board, piece::Piece, square::Square};
use crate::Result;

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
    pub fn distance(self) -> u32 {
        self.from.distance(self.to)
    }

    pub fn try_into_cmove(self, board: Board) -> Result<CMove> {
        let Self { from, to } = self;
        if let Some((piece, color)) = board.colored_piece_on(from) {
            let kind = match (color, piece, from, to) {
                (Color::White, Piece::King, Square::E1, Square::G1) => {
                    CMoveKind::Castles(CastleSide::Short)
                }
                (Color::White, Piece::King, Square::E1, Square::C1) => {
                    CMoveKind::Castles(CastleSide::Long)
                }
                (Color::Black, Piece::King, Square::E8, Square::G8) => {
                    CMoveKind::Castles(CastleSide::Short)
                }
                (Color::Black, Piece::King, Square::E8, Square::C8) => {
                    CMoveKind::Castles(CastleSide::Long)
                }
                _ => {
                    let captures = if let Some((_, captured_color)) = board.colored_piece_on(to) {
                        captured_color != board.side_to_move()
                    } else {
                        false
                    };
                    let promotion = match (piece, color, to.rank()) {
                        (Piece::Pawn, Color::Black, Rank::First)
                        | (Piece::Pawn, Color::White, Rank::Eighth) => Some(Piece::Queen),
                        _ => None,
                    };

                    let (src_rank, src_file) = if board.get_valid_moves_to(to, piece).len() > 1 {
                        (Some(from.rank()), Some(from.file()))
                    } else if piece == Piece::Pawn && captures {
                        (None, Some(from.file()))
                    } else {
                        (None, None)
                    };

                    CMoveKind::Regular(MoveDetails {
                        piece,
                        captures,
                        dst_rank: to.rank(),
                        dst_file: to.file(),
                        src_rank,
                        src_file,
                        promotion,
                    })
                }
            };
            let next_board = board.clone().update(self);
            let check = next_board.is_check();
            let checkmate = next_board.is_checkmate();
            return Ok(CMove {
                kind,
                check,
                color: board.side_to_move(),
                checkmate,
                comment: None,
            });
        };
        Err(Error::ParseError(ParseKind::MoveToCMove))
    }
    pub fn as_notation(self, board: &Board) -> String {
        let Move { from, to } = self;
        let piece = board.piece_on(from).unwrap();
        let mut move_text = String::new();
        if piece == Piece::King {
            if (from, to) == (Square::E1, Square::G1) || (from, to) == (Square::E8, Square::G8) {
                return String::from("0-0");
            } else if (from, to) == (Square::E1, Square::C1)
                || (from, to) == (Square::E8, Square::C8)
            {
                return String::from("0-0-0");
            }
        } else {
            let is_capture = board.piece_on(to).is_some();
            match piece {
                Piece::Pawn => {
                    move_text = if is_capture {
                        format!("{}x{}", from.file().as_str(), to)
                    } else {
                        format!("{to}")
                    }
                }
                _ => {
                    move_text = if is_capture {
                        format!("{piece}x{to}")
                    } else {
                        format!("{piece}{to}")
                    }
                }
            }
        }
        move_text
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.from, self.to)
    }
}
