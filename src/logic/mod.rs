mod movetree;

use std::str::FromStr;

use anyhow::Result;
use indextree::NodeId;

use crate::{
    common::{board::Board, r#move::Move, square::Square},
    error::Error,
    event::Event,
};

use self::movetree::{treenode::Notation, MoveTree};

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
                let new_node = self.play(from, to, displayed_node, self.board);
                let _ = self.sender.send(Event::NewNodeAppended(new_node));
            }
            Event::RequestBoard => {
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
                self.board = Board::from_str(self.move_tree.get()[node].get().fen.as_str())
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
        mut board: Board,
    ) -> Result<NodeId, Error> {
        let m = Move::new(from, to);
        if board.is_legal(m) {
            let new_node = self.move_tree.add_new_move(m, displayed_node, &board);
            self.board = board.update(m);
            return Ok(new_node);
        }
        Err(Error::IllegalMove)
    }

    pub fn prev_move(&mut self, node_id: NodeId) -> Result<NodeId, Error> {
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

    pub fn next_move(&mut self, node: Option<NodeId>) -> Result<NextMoveOptions, Error> {
        match node {
            Some(n) => match n.children(self.move_tree.get()).count() {
                0 => Err(Error::NoNextMove),
                1 => {
                    let child_node_id = n.children(self.move_tree.get()).nth(0).unwrap();
                    self.board = Board::from_str(self.move_tree.get_fen_for_node(child_node_id))
                        .expect("Failed to load board from next_move fen");
                    Ok(NextMoveOptions::Single(child_node_id))
                }
                _ => {
                    let options = n
                        .children(&self.move_tree.0)
                        .map(|child| (child, self.move_tree.get()[child].get().notation.clone()))
                        .collect();
                    Ok(NextMoveOptions::Multiple(options))
                }
            },
            None => {
                let roots = self.move_tree.get_tree_roots();
                match roots.len() {
                    0 => Err(Error::NoNextMove),
                    1 => {
                        let root = roots[0];
                        self.board = Board::from_str(self.move_tree.get_fen_for_node(root))
                            .expect("Failed to load board from next_move fen");
                        Ok(NextMoveOptions::Single(root))
                    }
                    _ => {
                        let options = roots
                            .into_iter()
                            .map(|child| {
                                (child, self.move_tree.get()[child].get().notation.clone())
                            })
                            .collect();
                        Ok(NextMoveOptions::Multiple(options))
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum NextMoveOptions {
    Single(NodeId),
    Multiple(Vec<(NodeId, Notation)>),
}
