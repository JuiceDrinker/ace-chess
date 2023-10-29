mod treenode;

use std::str::FromStr;

use indextree::{Arena, NodeId};

use crate::{
    common::{board::Board, r#move::Move, square::Square},
    event::Event,
};

use self::treenode::TreeNode;

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
                if new_node.is_some() {
                    let _ = self.sender.send(Event::NewNodeAppended(new_node));
                }
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
            _ => {}
        }
    }

    pub fn play(
        &mut self,
        from: Square,
        to: Square,
        displayed_node: Option<NodeId>,
        mut board: Board,
    ) -> Option<NodeId> {
        let m = Move::new(from, to);
        if board.is_legal(m) {
            let new_node = match displayed_node {
                None => {
                    let id = self.move_tree.new_node(TreeNode::new(&m, board));
                    Some(id)
                }
                Some(node) => {
                    match node
                        .children(&self.move_tree)
                        .find(|n| self.move_tree[*n].get().notation == m.as_notation(&board))
                    {
                        Some(child) => Some(child),
                        None => {
                            let id = self.move_tree.new_node(TreeNode::new(&m, board));
                            node.append(id, &mut self.move_tree);
                            Some(id)
                        }
                    }
                }
            };
            self.board = board.update(m);
            return new_node;
        }
        None
    }

    pub fn prev_move(&mut self, node: NodeId) -> Option<NodeId> {
        match node.ancestors(&self.move_tree).nth(1) {
            Some(prev_id) => {
                self.board = Board::from_str(self.move_tree[prev_id].get().fen.as_str())
                    .expect("Failed to load board from prev_move fen");
                Some(prev_id)
            }
            None => {
                self.board = Board::default();
                None
            }
        }
    }
}
