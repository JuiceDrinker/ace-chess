use crate::common::{r#move::Move, square::Square};
use indextree::NodeId;

#[derive(Clone, Debug)]
pub enum Message {
    SelectSquare(Square),
    MakeMove(Move, NodeId),
    // HideNextMoveOptions,
    GoPrevMove,
    GoNextMove,
    GoToNode(NodeId),
    InitLoadPgn,
    LoadPgn(String),
}
