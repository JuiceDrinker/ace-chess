pub mod pgn;
pub mod treenode;

use indextree::{Arena, NodeId};

use crate::{error::Error, Result};

use self::{
    pgn::parser::STARTING_POSITION_FEN,
    treenode::{CMove, Fen, Notation, TreeNode},
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

    pub fn get_fen_for_node(&self, id: NodeId) -> Option<&str> {
        match self.tree[id].get() {
            TreeNode::Move(fen, _) => Some(fen),
            _ => None,
        }
    }
}
