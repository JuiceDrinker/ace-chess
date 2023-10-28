use crate::common::{board::Board, square::Square};

#[derive(Debug)]
pub enum Event {
    MakeMove(Square, Square),
    RequestBoard,
    SendBoard(Board),
}
