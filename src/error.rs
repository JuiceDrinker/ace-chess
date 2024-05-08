#[derive(PartialEq, Debug, Clone)]
pub enum Error {
    Comm,
    InvalidRank,
    InvalidFile,
    InvalidSquare,
    InvalidFen { fen: String },
    IllegalMove,
    NoPrevMove,
    NoNextMove,
    OwnPieceOnSquare,
    InvalidPgn { pgn: String },
    ParseError(ParseKind),
    PgnGeneration,
}

#[derive(PartialEq, Debug, Clone)]
pub enum ParseKind {
    CharToPiece,
    StringToPgn,
    StringToNag,
}
