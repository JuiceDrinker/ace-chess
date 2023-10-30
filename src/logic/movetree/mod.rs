pub mod treenode;

use indextree::{Arena, NodeId};

use crate::{
    common::{board::Board, r#move::Move},
    error::Error,
};

use self::treenode::Notation;
// Dont expose this eventually..
pub use self::treenode::TreeNode;

#[derive(Debug, Clone)]
pub struct MoveTree(pub Arena<TreeNode>);

impl MoveTree {
    pub fn new() -> Self {
        MoveTree(Arena::new())
    }

    pub fn get_tree(&self) -> &Arena<TreeNode> {
        &self.0
    }

    pub fn get_tree_roots(&self) -> Vec<NodeId> {
        self.0
            .iter()
            .filter(|node| node.parent().is_none())
            .map(|node| self.0.get_node_id(node).unwrap())
            .collect()
    }

    pub fn get_fen_for_node(&self, id: NodeId) -> &str {
        self.0[id].get().fen.as_str()
    }

    pub fn get_prev_move(&self, id: NodeId) -> Result<(NodeId, &str), Error> {
        match id.ancestors(self.get_tree()).nth(1) {
            // 0th value is node itself    ^
            Some(prev_id) => Ok((prev_id, self.get_fen_for_node(prev_id))),
            None => Err(Error::NoPrevMove),
        }
    }

    pub fn get_next_move(&self, node: Option<NodeId>) -> Result<NextMoveOptions, Error> {
        match node {
            Some(n) => match n.children(self.get_tree()).count() {
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
            },
            None => {
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
    }

    pub fn add_new_move(
        &mut self,
        r#move: Move,
        displayed_node: Option<NodeId>,
        board: &Board,
    ) -> NodeId {
        match displayed_node {
            None => {
                // If displayed_node is none, we are in starting position
                // Look for roots, dont append if root with same move exists
                match self
                    .get_tree_roots()
                    .into_iter()
                    .find(|n| self.get_tree()[*n].get().notation == r#move.as_notation(board))
                {
                    Some(node) => node,
                    None => self.0.new_node(TreeNode::new(&r#move, board)),
                }
            }
            Some(node) => {
                match node
                    .children(self.get_tree())
                    .find(|n| self.get_tree()[*n].get().notation == r#move.as_notation(board))
                {
                    Some(child) => child,
                    None => {
                        let id = self.0.new_node(TreeNode::new(&r#move, board));
                        node.append(id, &mut self.0);
                        id
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum NextMoveOptions {
    Single(NodeId, String),
    Multiple(Vec<(NodeId, Notation)>),
}
