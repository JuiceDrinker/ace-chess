pub mod treenode;

use indextree::{Arena, NodeId};

use crate::common::{board::Board, r#move::Move};

// Dont expose this eventually..
pub use self::treenode::TreeNode;

#[derive(Debug, Clone)]
pub struct MoveTree(pub Arena<TreeNode>);

impl MoveTree {
    pub fn new() -> Self {
        MoveTree(Arena::new())
    }

    pub fn get(&self) -> &Arena<TreeNode> {
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
                    .find(|n| self.get()[*n].get().notation == r#move.as_notation(board))
                {
                    Some(node) => node,
                    None => self.0.new_node(TreeNode::new(&r#move, board)),
                }
            }
            Some(node) => {
                match node
                    .children(self.get())
                    .find(|n| self.get()[*n].get().notation == r#move.as_notation(board))
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
