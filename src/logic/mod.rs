pub mod movetree;

use std::str::FromStr;

use indextree::NodeId;

use crate::{
    common::{board::Board, r#move::Move, square::Square},
    error::Error,
    event::{Event, NextMoveResponse},
    prelude::Result,
};

use self::movetree::{MoveTree, NextMoveOptions};

#[derive(Debug, Clone)]
pub struct Dispatcher {
    move_tree: MoveTree,
    board: Board,
    sender: crossbeam_channel::Sender<Event>,
}
impl Dispatcher {
    pub fn new(board: Board, sender: crossbeam_channel::Sender<Event>) -> Self {
        Self {
            board,
            sender,
            move_tree: MoveTree::new(),
        }
    }

    pub fn dispatch(&mut self, event: Event) {
        match event {
            Event::MakeMove(from, to, displayed_node) => {
                let new_node = self.play(from, to, displayed_node);
                let _ = self.sender.send(Event::NewNodeAppended(new_node));
            }
            Event::GetBoard => {
                let _ = self.sender.send(Event::SendBoard(self.board));
            }
            Event::GetLegalMoves(square) => {
                let _ = self
                    .sender
                    .send(Event::SendLegalMoves(self.board.get_legal_moves(square)));
            }
            Event::GetPrevMove(displayed_node) => {
                let new_node = self.prev_move(displayed_node);
                let _ = self.sender.send(Event::NewDisplayNode(new_node));
            }
            Event::GetNextMove(displayed_node) => {
                let new_node = self.next_move(displayed_node);
                let _ = self.sender.send(Event::NextMoveResponse(new_node));
            }
            Event::GoToNode(node) => {
                self.board = Board::from_str(self.move_tree.get_tree()[node].get().fen.as_str())
                    .expect("Failed to load board from node fen");
                let _ = self.sender.send(Event::NewDisplayNode(Ok(node)));
            }
            _ => {}
        }
    }

    pub fn play(
        &mut self,
        from: Square,
        to: Square,
        displayed_node: Option<NodeId>,
    ) -> Result<NodeId> {
        let m = Move::new(from, to);
        if self.board.is_legal(m) {
            let new_node = self.move_tree.add_new_move(m, displayed_node, &self.board);
            self.board = self.board.update(m);
            return Ok(new_node);
        } else if self.board.color_on_is(to, self.board.side_to_move()) {
            return Err(Error::OwnPieceOnSquare);
        }
        Err(Error::IllegalMove)
    }

    pub fn prev_move(&mut self, node_id: NodeId) -> Result<NodeId> {
        match self.move_tree.get_prev_move(node_id) {
            Ok((id, fen)) => {
                self.board = Board::from_str(fen).expect("Failed to load board from prev_move fen");
                Ok(id)
            }
            Err(e) => {
                self.board = Board::default();
                Err(e)
            }
        }
    }

    pub fn next_move(&mut self, node: Option<NodeId>) -> Result<NextMoveResponse> {
        match self.move_tree.get_next_move(node) {
            Ok(NextMoveOptions::Single(id, fen)) => {
                self.board =
                    Board::from_str(&fen).expect("Failed to load board from next_move fen");
                Ok(NextMoveResponse::Single(id))
            }
            Ok(NextMoveOptions::Multiple(options)) => Ok(NextMoveResponse::Multiple(options)),
            Err(e) => Err(e),
        }
    }
}
