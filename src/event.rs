use indextree::NodeId;

use crate::{
    common::{board::Board, square::Square},
    error::Error,
    logic::NextMoveOptions,
};
use anyhow::Result;

#[derive(Clone, Debug)]
pub enum Event {
    NextMoveResponse(Result<NextMoveOptions, Error>),
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
