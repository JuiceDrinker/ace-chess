use crate::common::{board::Board, r#move::Move};

pub type Notation = String;
pub type Fen = String;

#[derive(Debug, PartialEq, Eq, Clone)]
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
}
