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
            CMoveKind::Castles(side) => {
                san = match side {
                    CastleSide::Short => "O-O".to_string(),
                    CastleSide::Long => "O-O-O".to_string(),
                };
            }
            CMoveKind::Regular(details) => {
                // Add piece symbol (except for pawns)
                if details.piece != Piece::Pawn {
                    san.push_str(&format!("{}", details.piece));
                }

                // Add disambiguation if necessary
                if let Some(file) = details.src_file {
                    san.push_str(file.as_str());
                }
                if let Some(rank) = details.src_rank {
                    san.push_str(&format!("{}", rank));
                }

                // Add capture symbol
                if details.captures {
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
    pub src_rank: Option<Rank>,
    pub src_file: Option<File>,
    pub promotion: Option<Piece>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CResult {
    WhiteWins,
    BlackWins,
    Draw,
    NoResult,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regular_move_pawn() {
        let cmove = CMove {
            kind: CMoveKind::Regular(MoveDetails {
                piece: Piece::Pawn,
                src_file: None,
                src_rank: None,
                dst_file: File::E,
                dst_rank: Rank::Fourth,
                captures: false,
                promotion: None,
            }),
            check: false,
            color: Color::White,
            checkmate: false,
            comment: None,
        };
        assert_eq!(cmove.to_san(), "e4");
    }

    #[test]
    fn test_regular_move_knight_with_disambiguation() {
        let cmove = CMove {
            kind: CMoveKind::Regular(MoveDetails {
                piece: Piece::Knight,
                src_file: Some(File::C),
                src_rank: None,
                dst_file: File::D,
                dst_rank: Rank::Fifth,
                captures: false,
                promotion: None,
            }),
            check: false,
            color: Color::White,
            checkmate: false,
            comment: None,
        };
        assert_eq!(cmove.to_san(), "Ncd5");
    }

    #[test]
    fn test_regular_move_capture_with_check() {
        let cmove = CMove {
            kind: CMoveKind::Regular(MoveDetails {
                piece: Piece::Queen,
                src_file: None,
                src_rank: None,
                dst_file: File::F,
                dst_rank: Rank::Seventh,
                captures: true,
                promotion: None,
            }),
            check: true,
            color: Color::Black,
            checkmate: false,
            comment: None,
        };
        assert_eq!(cmove.to_san(), "Qxf7+");
    }

    #[test]
    fn test_pawn_capture_with_promotion_and_checkmate() {
        let cmove = CMove {
            kind: CMoveKind::Regular(MoveDetails {
                piece: Piece::Pawn,
                src_file: Some(File::G),
                src_rank: None,
                dst_file: File::H,
                dst_rank: Rank::Eighth,
                captures: true,
                promotion: Some(Piece::Queen),
            }),
            check: false,
            color: Color::White,
            checkmate: true,
            comment: None,
        };
        assert_eq!(cmove.to_san(), "gxh8=Q#");
    }

    #[test]
    fn test_castles_kingside() {
        let cmove = CMove {
            kind: CMoveKind::Castles(CastleSide::Short),
            check: false,
            color: Color::White,
            checkmate: false,
            comment: None,
        };
        assert_eq!(cmove.to_san(), "O-O");
    }

    #[test]
    fn test_castles_queenside_with_check() {
        let cmove = CMove {
            kind: CMoveKind::Castles(CastleSide::Long),
            check: true,
            color: Color::Black,
            checkmate: false,
            comment: None,
        };
        assert_eq!(cmove.to_san(), "O-O-O+");
    }

    #[test]
    fn test_move_with_comment() {
        let cmove = CMove {
            kind: CMoveKind::Regular(MoveDetails {
                piece: Piece::Bishop,
                src_file: None,
                src_rank: None,
                dst_file: File::C,
                dst_rank: Rank::Fourth,
                captures: false,
                promotion: None,
            }),
            check: false,
            color: Color::White,
            checkmate: false,
            comment: Some("Good move!".to_string()),
        };
        assert_eq!(cmove.to_san(), "Bc4 Good move!");
    }
}
