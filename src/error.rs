#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidRank,
    InvalidFile,
    InvalidSquare,
    InvalidFen { fen: String },
}
