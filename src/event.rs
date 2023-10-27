use crate::common::square::Square;

#[derive(Debug)]
pub enum Event {
    MakeMove(Square, Square),
    AskForBoard,
}
