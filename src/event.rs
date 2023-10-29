use indextree::NodeId;

use crate::common::{board::Board, square::Square};

#[derive(Clone, Debug)]
pub enum Event {
    NewDisplayNode(Option<NodeId>),
    MakeMove(Square, Square, Option<NodeId>),
    RequestBoard,
    SendBoard(Board),
    GetLegalMoves(Square),
    SendLegalMoves(Vec<Square>),
    GetPrevMove(NodeId),
    NewNodeAppended(Option<NodeId>),
}
