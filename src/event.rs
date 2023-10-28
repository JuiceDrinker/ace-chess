use crate::common::{board::Board, square::Square};

#[derive(Clone, Debug)]
pub enum Event {
    MakeMove(Square, Square),
    RequestBoard,
    SendBoard(Board),
    GetLegalMoves(Square),
    SendLegalMoves(Vec<Square>),
    GetPrevMove,
}
