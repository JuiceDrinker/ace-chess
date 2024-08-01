use crate::common::square::Square;
use indextree::NodeId;

#[derive(Clone, Debug)]
pub enum Message {
    SelectSquare(Square),
    MakeMove(Square, Square, Option<NodeId>),
    // HideNextMoveOptions,
    GoPrevMove,
    GoNextMove,
    GoToNode(NodeId),
    LoadPgn(String),
}
