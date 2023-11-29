use crate::{
    common::{board::Board, square::Square},
    logic::movetree::treenode::Notation,
    prelude::Result,
};
use indextree::NodeId;

#[derive(Clone, Debug)]
pub enum Event {
    NextMoveResponse(Result<NextMoveResponse>),
    NewDisplayNode(Result<NodeId>),
    MakeMove(Square, Square, Option<NodeId>),
    GetBoard,
    SendBoard(Board),
    GetLegalMoves(Square),
    SendLegalMoves(Vec<Square>),
    GetPrevMove(NodeId),
    GetNextMove(Option<NodeId>),
    NewNodeAppended(Result<NodeId>),
    GoToNode(NodeId),
    LoadPgn(String),
}

#[derive(Clone, Debug)]
pub enum NextMoveResponse {
    Single(NodeId),
    Multiple(Vec<(NodeId, Notation)>),
}
