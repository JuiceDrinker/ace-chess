use crate::{
    common::{board::Board, r#move::Move, square::Square},
    event::Event,
};

#[derive(Debug, Clone)]
pub struct Dispatcher {
    board: Board,
    sender: crossbeam_channel::Sender<Board>,
}
impl Dispatcher {
    pub fn new(board: Board, sender: crossbeam_channel::Sender<Board>) -> Self {
        Self { board, sender }
    }

    pub fn dispatch(&mut self, event: Event) {
        match event {
            Event::MakeMove(from, to) => {
                self.play(from, to, self.board);
            }
            Event::RequestBoard => {
                let _ = self.sender.send(self.board);
            }
        }
    }

    pub fn play(&mut self, from: Square, to: Square, mut board: Board) {
        let m = Move::new(from, to);
        if board.is_legal(m) {
            //     match self.board.displayed_node {
            //         None => {
            //             let id = self.tree.new_node(TreeNode::new(&m, self));
            //             self.board.displayed_node = Some(id);
            //         }
            //         Some(node) => {
            //             match node
            //                 .children(&self.tree)
            //                 .find(|n| self.tree[*n].get().notation == self.convert_move_to_notation(&m))
            //             {
            //                 Some(child) => self.board.displayed_node = Some(child),
            //                 None => {
            //                     let new_node = self.tree.new_node(TreeNode::new(&m, self));
            //                     node.append(new_node, &mut self.tree);
            //                     self.board.displayed_node = Some(new_node);
            //                 }
            //             }
            //         }
            //     }

            self.board = board.update(m);
        }
        // }
    }
}
