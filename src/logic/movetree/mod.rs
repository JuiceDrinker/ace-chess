pub mod pgn;
pub mod treenode;

use std::str::FromStr;

use indextree::{Arena, NodeId};

use crate::{
    common::{board::Board, color::Color, square::Square},
    error::Error,
    Result,
};

use self::{
    pgn::parser::{Expression, STARTING_POSITION_FEN},
    treenode::{CMove, CMoveKind, CastleSide, Fen, Notation, TreeNode},
};

#[derive(Clone, Debug)]
pub enum NextMoveOptions {
    Single(NodeId, Fen),
    Multiple(Vec<(NodeId, Notation)>),
}
impl NextMoveOptions {
    pub fn new(options: Vec<(NodeId, Fen, Notation)>) -> Result<Self> {
        match options.len() {
            0 => Err(Error::NoNextMove),
            1 => {
                let (id, fen, _) = options.first().cloned().unwrap();
                Ok(NextMoveOptions::Single(id, fen))
            }
            _ => Ok(NextMoveOptions::Multiple(
                options
                    .into_iter()
                    .map(|(id, _, notation)| (id, notation))
                    .collect(),
            )),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MoveTree {
    tree: Arena<TreeNode>,
    game_start: NodeId,
}

impl Default for MoveTree {
    fn default() -> Self {
        MoveTree::new()
    }
}

impl MoveTree {
    pub fn new() -> Self {
        let mut tree: indextree::Arena<TreeNode> = indextree::Arena::new();
        let game_start = tree.new_node(TreeNode::GameStart);

        Self { tree, game_start }
    }

    pub fn add_new_move(&mut self, new_cmove: CMove, parent: NodeId, new_fen: String) -> NodeId {
        // Check for duplicate moves among the children of the parent node
        let duplicate =
            parent
                .children(&self.tree)
                .find_map(|child| match self.tree[child].get() {
                    TreeNode::StartVariation => {
                        let parent = self.tree[child].parent().unwrap();
                        match self.tree[parent].get() {
                            TreeNode::Move(_, cmove) if *cmove == new_cmove => Some(parent),
                            _ => None,
                        }
                    }
                    TreeNode::Move(_, cmove) if *cmove == new_cmove => Some(child),
                    _ => None,
                });

        match duplicate {
            Some(id) => id,
            None => {
                let node = self.tree.new_node(TreeNode::Move(new_fen, new_cmove));
                parent.append(node, &mut self.tree);
                node
            }
        }
    }

    fn add_move_to_tree(&mut self, cmove: CMove, parent: indextree::NodeId) -> indextree::NodeId {
        if let Ok(fen) = generate_next_fen(&self.get_last_fen(parent), &cmove) {
            let new_node = self.tree.new_node(TreeNode::Move(fen.clone(), cmove));
            parent.append(new_node, &mut self.tree);
            new_node
        } else {
            match self.tree[parent].parent() {
                Some(grandparent) => {
                    let fen = self.get_last_fen(grandparent);
                    let new_node = self.tree.new_node(TreeNode::Move(fen.clone(), cmove));
                    parent.append(new_node, &mut self.tree);
                    new_node
                }
                None => panic!("wtf something went wrong adding to tree"),
            }
        }
    }

    fn add_expression_to_tree(
        &mut self,
        expression: Expression,
        parent: indextree::NodeId,
    ) -> indextree::NodeId {
        match expression {
            Expression::Move(cmove) => self.add_move_to_tree(cmove, parent),
            Expression::Variation(expressions) => self.add_variation_to_tree(expressions, parent),
        }
    }

    fn add_variation_to_tree(
        &mut self,
        expressions: Vec<Expression>,
        parent: indextree::NodeId,
    ) -> indextree::NodeId {
        let grand_parent = self.tree[parent].parent().unwrap();
        let start_variation = self.tree.new_node(TreeNode::StartVariation);
        dbg!(self.tree[parent].get());
        grand_parent.append(start_variation, &mut self.tree);

        let mut var_current = start_variation;

        for expression in expressions {
            let new_node = self.add_expression_to_tree(expression, var_current);
            var_current = if let TreeNode::EndVariation = self.tree[new_node].get() {
                parent
            } else {
                new_node
            }
        }

        let end_variation = self.tree.new_node(TreeNode::EndVariation);
        var_current.append(end_variation, &mut self.tree);

        end_variation
    }

    fn add_sequence_to_tree(
        &mut self,
        first: Expression,
        second: Expression,
        parent: indextree::NodeId,
    ) -> indextree::NodeId {
        // dbg!(first.clone(), second.clone());
        let new_node = self.add_expression_to_tree(first, parent);
        // dbg!(self.tree[parent].get());
        // dbg!(self.tree[new_node].get());
        if let Expression::Variation(_) = second {
            self.add_expression_to_tree(second, parent)
        } else {
            self.add_expression_to_tree(second, new_node)
        }
    }

    // If given node has FEN, return it
    // else traverse up tree till you find one
    fn get_last_fen(&self, node: indextree::NodeId) -> Fen {
        if let TreeNode::Move(fen, _) = self.tree[node].get() {
            return fen.clone();
        } else {
            let mut current = node;
            while let Some(parent) = self.tree.get(current).unwrap().parent() {
                if let TreeNode::Move(fen, _) = self.tree[parent].get() {
                    return fen.clone();
                }
                current = parent;
            }
        }
        // If we can't find a parent move, return the starting position
        STARTING_POSITION_FEN.to_string()
    }

    pub fn get_fen_for_node(&self, id: NodeId) -> Option<&str> {
        match self.tree[id].get() {
            TreeNode::Move(fen, _) => Some(fen),
            _ => None,
        }
    }

    pub fn get_prev_move(&self, id: NodeId) -> (NodeId, Fen) {
        match id.ancestors(&self.tree).nth(1) {
            Some(parent_id) => match self.tree[parent_id].get() {
                TreeNode::GameStart => (self.game_start(), STARTING_POSITION_FEN.to_string()),
                TreeNode::StartVariation => self.get_prev_move(parent_id),
                TreeNode::Move(fen, _) => (parent_id, fen.to_string()),
                TreeNode::EndVariation | TreeNode::Result(_) => unreachable!(),
            },
            None => (self.game_start(), STARTING_POSITION_FEN.to_string()),
        }
    }

    pub fn get_next_move(&self, node: NodeId) -> Vec<(NodeId, Fen, Notation)> {
        node.children(&self.tree).fold(
            Vec::with_capacity(self.tree.capacity()),
            |mut acc, child| {
                match self.tree[child].get() {
                    TreeNode::StartVariation => acc.extend(self.get_next_move(child)),
                    TreeNode::Move(fen, cmove) => {
                        acc.push((child, fen.to_string(), cmove.to_san()))
                    }
                    TreeNode::EndVariation | TreeNode::GameStart | TreeNode::Result(_) => (),
                }
                acc
            },
        )
    }

    pub fn game_start(&self) -> NodeId {
        self.game_start
    }
}

fn generate_next_fen(current_fen: &str, cmove: &CMove) -> Result<Fen> {
    // NOTE: Currently board struct only handles promotion to queen
    let mut board = Board::from_str(current_fen).expect("To be able to create a valid fen");

    match &cmove.kind {
        CMoveKind::Castles(side) => {
            let (from, to) = match (cmove.color, side) {
                (Color::White, CastleSide::Short) => (Square::E1, Square::G1),
                (Color::White, CastleSide::Long) => (Square::E1, Square::C1),
                (Color::Black, CastleSide::Short) => (Square::E8, Square::G8),
                (Color::Black, CastleSide::Long) => (Square::E8, Square::C8),
            };
            Ok(board
                .update(crate::common::r#move::Move { from, to })
                .to_string())
        }
        CMoveKind::Regular(details) => {
            let dest = Square::make_square(details.dst_file, details.dst_rank);
            let potential_source_squares = board.get_valid_moves_to(dest, details.piece);

            if potential_source_squares.len() == 1 {
                board.update(crate::common::r#move::Move {
                    from: potential_source_squares.into_iter().next().unwrap(),
                    to: dest,
                });

                Ok(board.to_string())
            } else {
                // Handle disambiguation
                let mut from_square = None;
                for square in potential_source_squares {
                    if (details.src_file.is_some() && square.file() == details.src_file.unwrap())
                        || (details.src_rank.is_some()
                            && square.rank() == details.src_rank.unwrap())
                    {
                        from_square = Some(square);
                        break;
                    }
                }

                if let Some(from) = from_square {
                    board.update(crate::common::r#move::Move { from, to: dest });
                } else {
                    return Err(Error::FenGeneration {
                        fen: board.to_string(),
                        cmove: cmove.clone(),
                    });
                }
                Ok(board.to_string())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        common::{color::Color, file::File, piece::Piece, rank::Rank},
        logic::movetree::{
            generate_next_fen,
            pgn::parser::STARTING_POSITION_FEN,
            treenode::{CMove, CMoveKind, MoveDetails},
        },
    };

    #[test]
    fn next_fen() {
        let res = generate_next_fen(
            STARTING_POSITION_FEN,
            &CMove {
                kind: CMoveKind::Regular(MoveDetails {
                    piece: Piece::Pawn,
                    dst_rank: Rank::Fourth,
                    dst_file: File::D,
                    captures: false,
                    src_rank: None,
                    src_file: None,
                    promotion: None,
                }),
                check: false,
                color: Color::White,
                checkmate: false,
                comment: None,
            },
        );
        assert_eq!(
            res,
            Ok("rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq - 0 1".to_string())
        );
    }
}
// impl MoveTree {
//     pub fn load(&mut self, graph: Arena<TreeNode>) {
//         self.tree = graph;
//     }
//     pub fn get_tree(&self) -> &Arena<TreeNode> {
//         &self.tree
//     }
//
//
//     //
//     // pub fn get_color_for_node(&self, id: NodeId) -> &Color {
//     //     &self.tree[id].get().color
//     // }
//     // pub fn get_move_number_for_node(&self, id: NodeId) -> &usize {
//     //     &self.tree[id].get().move_number
//     // }
//     // pub fn get_notation_for_node(&self, id: NodeId) -> &str {
//     //     &self.tree[id].get().notation
//     // }
//
//
//     pub fn generate_pgn(&self) -> String {
//         match self.get_tree_roots().first() {
//             Some(root) => {
//                 let mut pgn: Vec<String> = vec![];
//                 let mut stack: Vec<(NodeId, Vec<String>)> = vec![(*root, vec![])];
//
//                 while let Some((node, mut path)) = stack.pop() {
//                     match self.get_color_for_node(node) {
//                         Color::Black => {
//                             path.push(format!(" {} ", self.get_notation_for_node(node)))
//                         }
//                         Color::White => path.push(format!(
//                             "{}. {} ",
//                             self.get_move_number_for_node(node),
//                             self.get_notation_for_node(node)
//                         )),
//                     };
//                     if !self.does_node_have_children(&node) {
//                         if self.is_node_mainline(&node) || self.is_node_root(&node) {
//                             pgn.push(path.join(""));
//                         } else {
//                             pgn.push(format!("( {} )", path.join("")));
//                         }
//                     } else {
//                         node.children(self.get_tree()).for_each(|child| {
//                             if self.is_node_mainline(&child) {
//                                 stack.push((child, path.clone()))
//                             } else {
//                                 stack.push((child, vec![]))
//                             }
//                         });
//                     }
//                 }
//                 pgn.join("")
//             }
//             None => "".to_string(),
//         }
//     }
//
//     pub fn does_node_have_children(&self, id: &NodeId) -> bool {
//         self.get_children_for_node(id).next().is_some()
//     }
//
//     pub fn is_node_mainline(&self, id: &NodeId) -> bool {
//         if let Some(parent) = self.get_tree()[*id].parent() {
//             return self.get_tree()[parent].get().depth == self.get_tree()[*id].get().depth;
//         }
//         false
//     }
//
//     pub fn is_node_root(&self, id: &NodeId) -> bool {
//         self.get_tree()[*id].parent().is_none()
//     }
//
//     pub fn get_children_for_node(&self, id: &NodeId) -> indextree::Children<'_, TreeNode> {
//         id.children(self.get_tree())
//     }
//
//
// }
