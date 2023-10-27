use std::sync::{Arc, RwLock};

use crate::{
    common::{board::Board, r#move::Move, square::Square},
    event::Event,
};

use self::writer::Writer;

mod writer;

pub fn dispatcher(event: Event, board: Arc<RwLock<Board>>) {
    dbg!("Got event: {:?}", &event);
    match event {
        Event::MakeMove(from, to) => play(from, to, board),
    }
}

pub fn play(from: Square, to: Square, board: Arc<RwLock<Board>>) {
    let writer = Writer::new(board.clone());
    let m = Move::new(from, to);
    if board.read().unwrap().is_legal(m) {
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

        writer.update_board(m);
    }
    // }
}
