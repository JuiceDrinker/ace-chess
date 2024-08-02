pub mod pgn;
pub mod treenode;

use std::{collections::HashSet, str::FromStr};

use crate::{
    common::{board::Board, color::Color, r#move::Move},
    error::Error,
    prelude::Result,
};
use indextree::{Arena, NodeId};

use self::treenode::{Notation, TreeNode};
// Dont expose this eventually..

#[derive(Clone, Debug)]
pub enum NextMoveOptions {
    Single(NodeId, String),
    Multiple(Vec<(NodeId, Notation)>),
}

#[derive(Debug, Default, Clone)]
pub struct MoveTree(pub Arena<TreeNode>);

// impl MoveTree {
//     pub fn load(&mut self, graph: Arena<TreeNode>) {
//         self.0 = graph;
//     }
//     pub fn get_tree(&self) -> &Arena<TreeNode> {
//         &self.0
//     }
//
//     pub fn get_tree_roots(&self) -> Vec<NodeId> {
//         self.get_tree()
//             .iter()
//             .filter(|node| node.parent().is_none())
//             .map(|node| self.get_tree().get_node_id(node).unwrap())
//             .collect()
//     }
//
//     // pub fn get_fen_for_node(&self, id: NodeId) -> &str {
//     //     self.0[id].get().fen.as_str()
//     // }
//     //
//     // pub fn get_color_for_node(&self, id: NodeId) -> &Color {
//     //     &self.0[id].get().color
//     // }
//     // pub fn get_move_number_for_node(&self, id: NodeId) -> &usize {
//     //     &self.0[id].get().move_number
//     // }
//     // pub fn get_notation_for_node(&self, id: NodeId) -> &str {
//     //     &self.0[id].get().notation
//     // }
//
//     pub fn get_prev_move(&self, id: NodeId) -> Result<(NodeId, &str)> {
//         match id.ancestors(self.get_tree()).nth(1) {
//             // 0th value is node itself    ^
//             Some(prev_id) => Ok((prev_id, self.get_fen_for_node(prev_id))),
//             None => Err(Error::NoPrevMove),
//         }
//     }
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
//     pub fn get_next_move(&self, node: Option<NodeId>) -> Result<NextMoveOptions> {
//         if let Some(n) = node {
//             match n.children(self.get_tree()).count() {
//                 0 => Err(Error::NoNextMove),
//                 1 => {
//                     let child_node_id = n.children(self.get_tree()).nth(0).unwrap();
//                     Ok(NextMoveOptions::Single(
//                         child_node_id,
//                         self.get_fen_for_node(child_node_id).to_string(),
//                     ))
//                 }
//                 _ => {
//                     let options = n
//                         .children(self.get_tree())
//                         .map(|child| (child, self.get_tree()[child].get().notation.clone()))
//                         .collect();
//                     Ok(NextMoveOptions::Multiple(options))
//                 }
//             }
//         } else {
//             let roots = self.get_tree_roots();
//             match roots.len() {
//                 0 => Err(Error::NoNextMove),
//                 1 => {
//                     let root = roots[0];
//                     Ok(NextMoveOptions::Single(
//                         root,
//                         self.get_fen_for_node(root).to_string(),
//                     ))
//                 }
//                 _ => {
//                     let options = roots
//                         .into_iter()
//                         .map(|child| (child, self.get_tree()[child].get().notation.clone()))
//                         .collect();
//                     Ok(NextMoveOptions::Multiple(options))
//                 }
//             }
//         }
//     }
//
//     pub fn add_new_move(&mut self, r#move: Move, parent: Option<NodeId>, board: &Board) -> NodeId {
//         match parent {
//             None => {
//                 // If displayed_node is none, we are in starting position
//                 // Look for roots, dont append if root with same move exists
//                 match self
//                     .get_tree_roots()
//                     .into_iter()
//                     .find(|n| self.get_tree()[*n].get().notation == r#move.as_notation(board))
//                 {
//                     Some(node) => node,
//                     None => self.0.new_node(TreeNode::new(
//                         r#move.as_notation(board),
//                         board.clone().update(r#move).to_string(),
//                         0,
//                         1,
//                         Color::White,
//                     )),
//                 }
//             }
//             Some(parent_node) => {
//                 // if move already exists in tree, don't duplicate
//                 if let Some(child) = parent_node
//                     .children(self.get_tree())
//                     .find(|n| self.get_tree()[*n].get().notation == r#move.as_notation(board))
//                 {
//                     child
//                 } else {
//                     // If my parent already has a child I should have depth +=1 of my parent
//                     // If I am first child I should have same depth as my parent
//                     let color = board.side_to_move();
//                     let move_number = board.fullmoves();
//                     let id = self.0.new_node(TreeNode::new(
//                         r#move.as_notation(board),
//                         board.clone().update(r#move).to_string(),
//                         match parent_node.children(self.get_tree()).count() > 0 {
//                             true => self.get_tree()[parent_node].get().depth + 1,
//                             false => self.get_tree()[parent_node].get().depth,
//                         },
//                         move_number.try_into().unwrap(),
//                         color,
//                     ));
//                     parent_node.append(id, &mut self.0);
//                     id
//                 }
//             }
//         }
//     }
// }
