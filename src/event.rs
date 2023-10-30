use crate::{
    common::{board::Board, square::Square},
    error::Error,
    logic::movetree::treenode::Notation,
};
use anyhow::Result;
use indextree::NodeId;

#[derive(Clone, Debug)]
pub enum Event {
    NextMoveResponse(Result<NextMoveResponse, Error>),
    NewDisplayNode(Result<NodeId, Error>),
    MakeMove(Square, Square, Option<NodeId>),
    RequestBoard,
    SendBoard(Board),
    GetLegalMoves(Square),
    SendLegalMoves(Vec<Square>),
    GetPrevMove(NodeId),
    GetNextMove(Option<NodeId>),
    NewNodeAppended(Result<NodeId, Error>),
    GoToNode(NodeId),
}

#[derive(Clone, Debug)]
pub enum NextMoveResponse {
    Single(NodeId),
    Multiple(Vec<(NodeId, Notation)>),
}
