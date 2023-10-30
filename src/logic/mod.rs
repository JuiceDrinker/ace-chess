mod treenode;

use std::str::FromStr;

use anyhow::Result;
use indextree::{Arena, NodeId};

use crate::{
    common::{board::Board, r#move::Move, square::Square},
    error::Error,
    event::Event,
};

use self::treenode::{Notation, TreeNode};

#[derive(Debug, Clone)]
pub struct Dispatcher {
    move_tree: Arena<TreeNode>,
    board: Board,
    sender: crossbeam_channel::Sender<Event>,
}
impl Dispatcher {
    pub fn new(board: Board, sender: crossbeam_channel::Sender<Event>) -> Self {
        Self {
            board,
            sender,
            move_tree: Arena::new(),
        }
    }

    pub fn dispatch(&mut self, event: Event) {
        match event {
            Event::MakeMove(from, to, displayed_node) => {
                // If move was illegal then new_node is None
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
            let new_node = match displayed_node {
                // If displayed_node is none, tree has no root/board is in starting position
                // TODO: If in starting position search for nodes with no parent and assume
                // those are roots
                None => self.move_tree.new_node(TreeNode::new(&m, board)),
                Some(node) => {
                    match node
                        .children(&self.move_tree)
                        .find(|n| self.move_tree[*n].get().notation == m.as_notation(&board))
                    {
                        Some(child) => child,
                        None => {
                            let id = self.move_tree.new_node(TreeNode::new(&m, board));
                            node.append(id, &mut self.move_tree);
                            id
                        }
                    }
                }
            };
            self.board = board.update(m);
            return Ok(new_node);
        }
        Err(Error::IllegalMove)
    }

    pub fn prev_move(&mut self, node: NodeId) -> Result<NodeId, Error> {
        match node.ancestors(&self.move_tree).nth(1) {
            Some(prev_id) => {
                self.board = Board::from_str(self.move_tree[prev_id].get().fen.as_str())
                    .expect("Failed to load board from prev_move fen");
                Ok(prev_id)
            }
            None => {
                self.board = Board::default();
                Err(Error::NoPrevMove)
            }
        }
    }

    pub fn next_move(&mut self, node: Option<NodeId>) -> Result<NextMoveOptions, Error> {
        match node {
            Some(n) => {
                // TODO: Currently automatically always choses first child
                // Give user option to choose one of the children
                match n.children(&self.move_tree).count() {
                    0 => Err(Error::NoNextMove),
                    1 => {
                        // let child_node_id = &self.move_tree[n].first_child().unwrap();
                        let child_node_id = n.children(&self.move_tree).nth(0).unwrap();
                        self.board =
                            Board::from_str(self.move_tree[child_node_id].get().fen.as_str())
                                .expect("Failed to load board from next_move fen");
                        Ok(NextMoveOptions::Single(child_node_id))
                    }
                    _ => {
                        let options = n
                            .children(&self.move_tree)
                            .map(|child| (child, self.move_tree[child].get().notation.clone()))
                            .collect();
                        Ok(NextMoveOptions::Multiple(options))
                    }
                }
            }
            None => {
                println!("Starting position");
                unimplemented!()
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum NextMoveOptions {
    Single(NodeId),
    Multiple(Vec<(NodeId, Notation)>),
}
