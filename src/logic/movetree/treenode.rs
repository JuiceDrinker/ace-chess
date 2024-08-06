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
