#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    InvalidRank,
    InvalidFile,
    InvalidSquare,
    InvalidFen { fen: String },
}
