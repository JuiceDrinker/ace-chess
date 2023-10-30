use serde::Serialize;

use crate::common::{board::Board, r#move::Move};

pub type Notation = String;
pub type Fen = String;

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct TreeNode {
    pub notation: Notation,
    pub fen: Fen,
}

impl TreeNode {
    pub fn new(r#move: &Move, board: Board) -> Self {
        TreeNode {
            notation: r#move.as_notation(&board),
            fen: board.clone().update(*r#move).to_string(),
        }
    }
    #[allow(dead_code)]
    pub fn pretty_print(node: indextree::NodeId, tree: &indextree::Arena<TreeNode>) {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_indextree::Node::new(node, tree)).unwrap()
        );
    }
}
