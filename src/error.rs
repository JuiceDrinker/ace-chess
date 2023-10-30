#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]

pub enum Error {
    Comm,
    InvalidRank,
    InvalidFile,
    InvalidSquare,
    InvalidFen { fen: String },
    IllegalMove,
    NoPrevMove,
    NoNextMove,
}
