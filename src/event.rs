use indextree::NodeId;

use crate::{
    common::{board::Board, square::Square},
    error::Error,
};
use anyhow::Result;

#[derive(Clone, Debug)]
pub enum Event {
    NewDisplayNode(Result<NodeId, Error>),
    MakeMove(Square, Square, Option<NodeId>),
    RequestBoard,
    SendBoard(Board),
    GetLegalMoves(Square),
    SendLegalMoves(Vec<Square>),
    GetPrevMove(NodeId),
    GetNextMove(Option<NodeId>),
    NewNodeAppended(Result<NodeId, Error>),
}
