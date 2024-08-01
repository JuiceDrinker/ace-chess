pub mod pgn;
pub mod treenode;

use std::{collections::HashSet, str::FromStr};

use crate::{
    common::{board::Board, color::Color, r#move::Move},
    error::Error,
    prelude::Result,
};
use indextree::{Arena, NodeId};

use self::treenode::Notation;
// Dont expose this eventually..
pub use self::treenode::TreeNode;

#[derive(Clone, Debug)]
pub enum NextMoveOptions {
    Single(NodeId, String),
    Multiple(Vec<(NodeId, Notation)>),
}

#[derive(Debug, Default, Clone)]
pub struct MoveTree(pub Arena<TreeNode>);

impl MoveTree {
    pub fn load(&mut self, graph: Arena<TreeNode>) {
        self.0 = graph;
    }
    pub fn get_tree(&self) -> &Arena<TreeNode> {
        &self.0
    }

    pub fn get_tree_roots(&self) -> Vec<NodeId> {
        self.get_tree()
            .iter()
            .filter(|node| node.parent().is_none())
            .map(|node| self.get_tree().get_node_id(node).unwrap())
            .collect()
    }

    pub fn get_fen_for_node(&self, id: NodeId) -> &str {
        self.0[id].get().fen.as_str()
    }

    pub fn get_notation_for_node(&self, id: NodeId) -> &str {
        &self.0[id].get().notation
    }

    pub fn get_prev_move(&self, id: NodeId) -> Result<(NodeId, &str)> {
        match id.ancestors(self.get_tree()).nth(1) {
            // 0th value is node itself    ^
            Some(prev_id) => Ok((prev_id, self.get_fen_for_node(prev_id))),
            None => Err(Error::NoPrevMove),
        }
    }

    pub fn is_node_mainline(&self, id: NodeId) -> bool {
        if let Some(p) = self.get_tree().get(id).unwrap().parent() {
            return self.get_tree()[p].get().depth == self.get_tree()[id].get().depth;
        }
        false
    }

    pub fn generate_pgn_for_children(
        &self,
        id: NodeId,
        visited: &mut HashSet<NodeId>,
    ) -> Option<String> {
        self.generate_pgn_from_node(id.children(self.get_tree()).next().unwrap(), visited)
    }

    pub fn generate_pgn_for_siblings(&self, id: NodeId, visited: &mut HashSet<NodeId>) -> String {
        id.following_siblings(self.get_tree())
            .skip(1)
            .fold(String::from(""), |mut acc, sibling| {
                if let Some(pgn_for_sibling) = self.generate_pgn_from_node(sibling, visited) {
                    acc.push_str(&format!("( {} )", pgn_for_sibling));
                }
                acc
            })
    }

    pub fn get_next_move(&self, node: Option<NodeId>) -> Result<NextMoveOptions> {
        if let Some(n) = node {
            match n.children(self.get_tree()).count() {
                0 => Err(Error::NoNextMove),
                1 => {
                    let child_node_id = n.children(self.get_tree()).nth(0).unwrap();
                    Ok(NextMoveOptions::Single(
                        child_node_id,
                        self.get_fen_for_node(child_node_id).to_string(),
                    ))
                }
                _ => {
                    let options = n
                        .children(self.get_tree())
                        .map(|child| (child, self.get_tree()[child].get().notation.clone()))
                        .collect();
                    Ok(NextMoveOptions::Multiple(options))
                }
            }
        } else {
            let roots = self.get_tree_roots();
            match roots.len() {
                0 => Err(Error::NoNextMove),
                1 => {
                    let root = roots[0];
                    Ok(NextMoveOptions::Single(
                        root,
                        self.get_fen_for_node(root).to_string(),
                    ))
                }
                _ => {
                    let options = roots
                        .into_iter()
                        .map(|child| (child, self.get_tree()[child].get().notation.clone()))
                        .collect();
                    Ok(NextMoveOptions::Multiple(options))
                }
            }
        }
    }

    pub fn add_new_move(&mut self, r#move: Move, parent: Option<NodeId>, board: &Board) -> NodeId {
        match parent {
            None => {
                // If displayed_node is none, we are in starting position
                // Look for roots, dont append if root with same move exists
                match self
                    .get_tree_roots()
                    .into_iter()
                    .find(|n| self.get_tree()[*n].get().notation == r#move.as_notation(board))
                {
                    Some(node) => node,
                    None => self.0.new_node(TreeNode::new(
                        r#move.as_notation(board),
                        board.clone().update(r#move).to_string(),
                        0,
                    )),
                }
            }
            Some(parent_node) => {
                // if move already exists in tree, don't duplicate
                if let Some(child) = parent_node
                    .children(self.get_tree())
                    .find(|n| self.get_tree()[*n].get().notation == r#move.as_notation(board))
                {
                    child
                } else {
                    // If my parent already has a child I should have depth +=1 of my parent
                    // If I am first child I should have same depth as my parent
                    let id = self.0.new_node(TreeNode::new(
                        r#move.as_notation(board),
                        board.clone().update(r#move).to_string(),
                        match parent_node.children(self.get_tree()).count() > 0 {
                            true => self.get_tree()[parent_node].get().depth + 1,
                            false => self.get_tree()[parent_node].get().depth,
                        },
                    ));
                    parent_node.append(id, &mut self.0);
                    id
                }
            }
        }
    }
    pub fn does_node_have_children(&self, id: NodeId) -> bool {
        id.children(self.get_tree()).next().is_some()
    }
    pub fn does_node_have_siblings(&self, id: NodeId) -> bool {
        // Skipping first node because first element is this node itself
        id.following_siblings(self.get_tree()).nth(1).is_some()
    }
}

pub trait GeneratePgn {
    fn generate_pgn(&self) -> String;
    fn generate_pgn_from_node(&self, root: NodeId, visited: &mut HashSet<NodeId>)
        -> Option<String>;
    fn generate_pgn_for_node(&self, node: NodeId) -> String;
}

impl GeneratePgn for MoveTree {
    fn generate_pgn(&self) -> String {
        if let Some(root) = self.get_tree_roots().first().copied() {
            self.generate_pgn_from_node(root, &mut HashSet::new())
                .unwrap_or(String::from(""))
        } else {
            String::from("")
        }
    }

    fn generate_pgn_from_node(
        &self,
        root: NodeId,
        visited: &mut HashSet<NodeId>,
    ) -> Option<String> {
        // Node IDs (for now) are in insertion order, so we should be able to assume the first
        //  one is the mainline
        if HashSet::insert(visited, root) {
            match (
                self.does_node_have_children(root),
                self.does_node_have_siblings(root),
            ) {
                // Has children, has siblings
                (true, true) => {
                    let mut pgn_for_node = self.generate_pgn_for_node(root);
                    if self.is_node_mainline(root) {
                        pgn_for_node
                            .push_str(&self.generate_pgn_for_siblings(root, visited).to_string());
                    };
                    if let Some(pgn_for_children) = &self.generate_pgn_for_children(root, visited) {
                        pgn_for_node.push_str(&format!(" {} ", pgn_for_children));
                    }
                    Some(pgn_for_node)
                }
                // Only have children
                (true, false) => {
                    let mut pgn_for_node = self.generate_pgn_for_node(root);

                    if let Some(pgn_for_children) = &self.generate_pgn_for_children(root, visited) {
                        pgn_for_node.push_str(&format!(" {} ", pgn_for_children));
                    }
                    Some(pgn_for_node)
                }
                // No children, yes siblings
                (false, true) => {
                    let mut pgn_for_node = self.generate_pgn_for_node(root);
                    if self.is_node_mainline(root) {
                        pgn_for_node
                            .push_str(&self.generate_pgn_for_siblings(root, visited).to_string());
                    };

                    Some(pgn_for_node)
                }
                // No children, no siblings
                (false, false) => Some(self.generate_pgn_for_node(root)),
            }
        } else {
            None
        }
    }

    fn generate_pgn_for_node(&self, node: NodeId) -> String {
        if Board::from_str(self.get_fen_for_node(node))
            .unwrap()
            .side_to_move
            == Color::Black
        {
            format!(
                " {}. {}",
                self.get_tree().get(node).unwrap().get().get_full_moves(),
                self.get_notation_for_node(node)
            )
        } else {
            format!(" {} ", self.get_notation_for_node(node))
        }
    }
}
