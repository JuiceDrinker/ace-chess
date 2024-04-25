use crate::common::square::Square;
use iced::event::Event;
use indextree::NodeId;

#[derive(Clone, Debug)]
pub enum Message {
    Event(Event),
    SelectSquare(Square),
    MakeMove(Square, Square, Option<NodeId>),
    HideNextMoveOptions,
    GoPrevMove,
    GoNextMove,
    GoToNode(NodeId),
}
