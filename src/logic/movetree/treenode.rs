use std::fmt::Display;

use crate::common::color::Color;
use crate::common::file::File;
use crate::common::piece::Piece;
use crate::common::rank::Rank;

pub(crate) type Notation = String;
pub type Fen = String;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TreeNode {
    GameStart,
    StartVariation,
    EndVariation,
    Move(Fen, CMove),
    Result(CResult),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CMoveKind {
    Regular(MoveDetails),
    Castles(CastleSide),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CMove {
    pub kind: CMoveKind,
    pub check: bool,
    pub color: Color,
    pub checkmate: bool,
    pub comment: Option<String>,
}

impl CMove {
    pub fn to_san(&self) -> String {
        let mut san = String::new();

        match &self.kind {
            CMoveKind::Regular(details) => {
                // Add piece symbol (except for pawns)
                if details.piece != Piece::Pawn {
                    san.push_str(&format!("{}", details.piece));
                }

                // Add disambiguation if necessary
                if let Some(file) = details.disam_file {
                    san.push_str(file.as_str());
                }
                if let Some(rank) = details.disam_rank {
                    san.push_str(&format!("{}", rank));
                }

                // Add capture symbol
                if details.captures {
                    if details.piece == Piece::Pawn {
                        san.push_str(
                            details
                                .disam_file
                                .expect("pawn captures must have disambiguation for file")
                                .as_str(),
                        );
                    }
                    san.push('x');
                }

                // Add destination square
                san.push_str(details.dst_file.as_str());
                san.push_str(&format!("{}", details.dst_rank));

                // Add promotion if applicable
                if let Some(promotion_piece) = details.promotion {
                    san.push('=');
                    san.push_str(&format!("{}", promotion_piece));
                }
            }
            CMoveKind::Castles(side) => {
                san = match side {
                    CastleSide::Short => "O-O".to_string(),
                    CastleSide::Long => "O-O-O".to_string(),
                };
            }
        }

        // Add check or checkmate symbol
        if self.checkmate {
            san.push('#');
        } else if self.check {
            san.push('+');
        }

        // Add comment if present
        if let Some(comment) = &self.comment {
            san.push_str(&format!(" {}", comment));
        }

        san
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CastleSide {
    Short,
    Long,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MoveDetails {
    pub piece: Piece,
    pub dst_rank: Rank,
    pub dst_file: File,
    pub captures: bool,
    pub disam_rank: Option<Rank>,
    pub disam_file: Option<File>,
    pub promotion: Option<Piece>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CResult {
    WhiteWins,
    BlackWins,
    Draw,
    NoResult,
}
