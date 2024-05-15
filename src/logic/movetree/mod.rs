pub mod pgn;
pub mod treenode;

use std::{collections::HashSet, fmt::Display, hash::Hash, str::FromStr};

use indextree::{Arena, Node, NodeId};

use crate::{
    common::{board::Board, color::Color, r#move::Move},
    error::Error,
    prelude::Result,
};

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
}

pub trait GeneratePgn {
    fn generate_pgn(&self, opts: GenerationType) -> String;
    fn generate_pgn_from_node(&self, root_key: NodeId) -> String;
    fn generate_pgn_for_node(&self, node: NodeId) -> String;
}

pub enum GenerationType {
    WholeTree,
    FromNode(NodeId),
}
impl GeneratePgn for MoveTree {
    fn generate_pgn(&self, opts: GenerationType) -> String {
        // dbg!(self.get_tree());
        // if let Some(root) = root_key.or_else(|| self.get_tree_roots().first().copied()) {}
        match opts {
            GenerationType::WholeTree => {
                if let Some(root) = self.get_tree_roots().first().copied() {
                    self.generate_pgn_from_node(root)
                } else {
                    String::from("")
                }
            }
            GenerationType::FromNode(node) => self.generate_pgn_from_node(node),
        }
    }

    fn generate_pgn_from_node(&self, root: NodeId) -> String {
        // // Node IDs (for now) are in insertion order, so we should be able to assume the first
        // // one is the mainline
        if root.children(self.get_tree()).count() == 0 {
            self.generate_pgn_for_node(root)
        } else {
            let mut pgn = String::from("");
            let mut pgn_for_node = self.generate_pgn_for_node(root);
            let mut branches = root.children(self.get_tree());
            let mainline = branches.next().unwrap();
            for branch in branches {
                if Board::from_str(self.get_fen_for_node(branch))
                    .unwrap()
                    .side_to_move
                    == Color::White
                {
                    pgn.push_str(&format!(
                        " ({}... ",
                        self.get_tree().get(root).unwrap().get().get_full_moves()
                    ));
                } else {
                    pgn.push_str(&format!(
                        " ({}. ",
                        self.get_tree().get(root).unwrap().get().get_full_moves()
                    ));
                }
                pgn_for_node.push_str(&format!(
                    "{})",
                    &self.generate_pgn(GenerationType::FromNode(branch))
                ));
            }
            pgn_for_node.push_str(&format!(
                " {} ",
                &self.generate_pgn(GenerationType::FromNode(mainline))
            ));
            pgn.push_str(&pgn_for_node);
            pgn
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
