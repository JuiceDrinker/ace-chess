use std::str::FromStr;

use serde::Serialize;

use crate::common::{board::Board, color::Color};

pub(crate) type Notation = String;
pub type Fen = String;

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct TreeNode {
    pub notation: Notation,
    pub fen: Fen,
    pub depth: usize,
}

impl TreeNode {
    pub fn new(notation: Notation, fen: Fen, depth: usize) -> Self {
        TreeNode {
            notation,
            fen,
            depth,
        }
    }

    #[allow(dead_code)]
    pub fn pretty_print(node: indextree::NodeId, tree: &indextree::Arena<TreeNode>) {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_indextree::Node::new(node, tree)).unwrap()
        );
    }

    // TODO: This shouldn't be here
    pub fn get_full_moves(&self) -> String {
        let board = Board::from_str(&self.fen).unwrap();
        if board.side_to_move == Color::Black {
            self.fen.trim_end().chars().last().unwrap().to_string()
        } else {
            (self
                .fen
                .trim_end()
                .chars()
                .last()
                .unwrap()
                .to_string()
                .parse::<usize>()
                .unwrap()
                - 1)
            .to_string()
        }
    }
}
