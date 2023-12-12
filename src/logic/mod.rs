pub mod movetree;

use std::str::FromStr;

use indextree::NodeId;

use crate::{
    common::{board::Board, r#move::Move, square::Square},
    error::Error,
    message::NextMoveResponse,
    prelude::Result,
};

use self::movetree::{pgn::STARTING_POSITION_FEN, MoveTree, NextMoveOptions};

// pub fn play(from: Square, to: Square, displayed_node: Option<NodeId>) -> Result<NodeId> {
//     let m = Move::new(from, to);
//     if self.board.is_legal(m) {
//         let new_node = self.move_tree.add_new_move(m, displayed_node, &self.board);
//         self.board = self.board.update(m);
//         return Ok(new_node);
//     } else if self.board.color_on_is(to, self.board.side_to_move()) {
//         return Err(Error::OwnPieceOnSquare);
//     }
//     Err(Error::IllegalMove)
// }

// pub fn load_pgn(pgn: &str) -> NodeId {
//     let graph = movetree::pgn::Parser::new()
//         .parse(pgn)
//         //TODO: Handle error if invalid PGN
//         .expect("Invalid PGN");
//     let root_id = &graph.get_tree_root();
//     self.move_tree = MoveTree(graph.0);
//     // TODO: Get root of tree
//     self.board = Board::from_str(STARTING_POSITION_FEN).unwrap();
//     *root_id
// }

// pub fn prev_move(move_tree: MoveTree, node_id: NodeId) -> Result<NodeId> {
//     match move_tree.get_prev_move(node_id) {
//         Ok((id, fen)) => {
//             self.board = Board::from_str(fen).expect("Failed to load board from prev_move fen");
//             Ok(id)
//         }
//         Err(e) => {
//             self.board = Board::default();
//             Err(e)
//         }
//     }
// }
//
// pub fn next_move(node: Option<NodeId>) -> Result<NextMoveResponse> {
//     match self.move_tree.get_next_move(node) {
//         Ok(NextMoveOptions::Single(id, fen)) => {
//             self.board = Board::from_str(&fen).expect("Failed to load board from next_move fen");
//             Ok(NextMoveResponse::Single(id))
//         }
//         Ok(NextMoveOptions::Multiple(options)) => Ok(NextMoveResponse::Multiple(options)),
//         Err(e) => Err(e),
//     }
// }
