use serde::Serialize;

pub(crate) type Notation = String;
// Type this as &str?
pub type Fen = String;

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct TreeNode {
    pub notation: Notation,
    pub fen: Fen,
}

impl TreeNode {
    pub fn new(notation: Notation, fen: Fen) -> Self {
        TreeNode { notation, fen }
    }
    #[allow(dead_code)]
    pub fn pretty_print(node: indextree::NodeId, tree: &indextree::Arena<TreeNode>) {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_indextree::Node::new(node, tree)).unwrap()
        );
    }
}
