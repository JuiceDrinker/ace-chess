pub const STARTING_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

use std::{
    collections::{HashMap, VecDeque},
    marker::PhantomData,
    str::FromStr,
};

mod parsers;
use indextree::NodeId;
use nom::{IResult, Parser as NomParserTrait};
use nom_supreme::{error::ErrorTree, ParserExt};

use crate::{
    common::{board::Board, color::Color, file::File, piece::Piece, square::Square},
    error::{Error, ParseKind},
    logic::movetree::pgn::parsers::{comment, move_number, move_text, nag},
    prelude::Result,
};

use super::{treenode::Fen, TreeNode};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Nag {
    Good,
    Excellent,
    Poor,
    Blunder,
    Dubious,
    Interesting,
}

impl FromStr for Nag {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "!" => Ok(Self::Good),
            "!!" => Ok(Self::Excellent),
            "?" => Ok(Self::Poor),
            "??" => Ok(Self::Blunder),
            "?!" => Ok(Self::Dubious),
            "!?" => Ok(Self::Interesting),
            _ => Err(Error::ParseError(ParseKind::StringToNag)),
        }
    }
}

type MoveText<'a> = &'a str;
type Move<'a> = (MoveText<'a>, Option<Comment>, Option<Nag>);

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AdjacencyList(pub indextree::Arena<TreeNode>);
impl AdjacencyList {
    pub fn new() -> Self {
        Self(indextree::Arena::new())
    }

    pub fn load(mut self, tree: indextree::Arena<TreeNode>) -> Self {
        self.0 = tree;
        self
    }

    fn add_node(&mut self, node: TreeNode, parent: Option<NodeId>) -> NodeId {
        let new_id = self.0.new_node(node);
        if let Some(id) = parent {
            id.append(new_id, &mut self.0);
        }
        new_id
    }

    pub fn get_tree_root(&self) -> NodeId {
        self.0
            .iter()
            .filter(|node| node.parent().is_none())
            .map(|node| self.0.get_node_id(node).unwrap())
            .collect::<Vec<_>>()[0]
    }
    fn get_parent(&self, key: NodeId) -> Option<NodeId> {
        self.0[key].parent()
    }

    fn get_children(&self, key: NodeId) -> Vec<NodeId> {
        key.children(&self.0).collect::<Vec<_>>()
    }

    fn get_fen(&self, key: NodeId) -> &Fen {
        &self.0[key].get().fen
    }
}

type Comment = String;

#[derive(Default, Debug, Clone)]
struct Variation(VecDeque<NodeId>, HashMap<NodeId, usize>);

impl Variation {
    fn get_variation_count(&self, key: NodeId) -> usize {
        *self.1.get(&key).unwrap_or(&0)
    }

    fn pop(&mut self) -> Option<NodeId> {
        if let Some(key) = self.0.pop_back() {
            if let Some(count) = self.1.get(&key) {
                if count > &1 {
                    self.1.entry(key).and_modify(|c| *c -= 1);
                } else {
                    self.1.remove_entry(&key);
                }
            }
            return Some(key);
        }
        None
    }

    fn push(&mut self, key: NodeId) {
        self.0.push_back(key);
        self.1
            .entry(key)
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    fn peek(&self) -> Option<&NodeId> {
        self.0.back()
    }
}

#[derive(Debug, Clone)]
pub struct Parser<'a> {
    stack: Variation,
    graph: AdjacencyList,
    current_key: NodeId,
    prev_node: NodeId,
    marker: PhantomData<&'a str>,
}

impl<'a> Parser<'a> {
    pub fn new() -> Self {
        let graph = AdjacencyList::new();
        let root = graph.get_tree_root();
        Self {
            stack: Variation::default(),
            graph,
            current_key: root,
            prev_node: root,
            marker: PhantomData,
        }
    }

    fn add_node(&mut self, (move_text, _, _): Move, parent: Option<NodeId>) -> NodeId {
        let starting_position = STARTING_POSITION_FEN.to_string();
        let prev_fen = match parent {
            Some(id) => self.graph.get_fen(id),
            None => &starting_position,
        };
        let next_fen = Parser::generate_next_fen(prev_fen, move_text);
        let id = self
            .graph
            .add_node(TreeNode::new(move_text.to_owned(), next_fen), parent);
        if let Some(parent_id) = parent {
            parent_id.append(id, &mut self.graph.0);
        }
        self.current_key = id;
        self.prev_node = self.current_key;
        id
    }

    pub fn parse(mut self, input: &'a str) -> Result<AdjacencyList> {
        let mut left_to_parse = input;
        while !left_to_parse.trim_start().is_empty() {
            // TODO: Make return type nicer since we never use second arg
            if let Ok((rest, _)) = self.move_entry(left_to_parse.trim_start()) {
                left_to_parse = rest;
            } else {
                return Err(Error::ParseError(ParseKind::StringToPgn));
            }
        }
        Ok(self.graph)
    }

    fn get_variation_creator(&self) -> Option<&NodeId> {
        self.stack.peek()
    }

    // Base case
    fn move_entry(&mut self, input: &'a str) -> IResult<&'a str, &'a str, ErrorTree<&'a str>> {
        let mut left_to_parse = input.trim_start();
        let mut parent_key: Option<NodeId> = None;
        if left_to_parse.starts_with("1-0")
            || input.trim_start().starts_with("1-0")
            || input.trim_start().starts_with("1/2-1/2")
        {
            // Eats rest of input
            // TODO: Keep on processing file if there are more games
            return Ok(("", ""));
        }
        if let Some(after_start_of_variation) = left_to_parse.strip_prefix('(') {
            // We are in first move of variation
            let parent = self.graph.get_parent(self.prev_node).unwrap();
            self.stack.push(parent);
            parent_key = Some(parent);
            left_to_parse = after_start_of_variation;
        } else if let Some(after_end_of_variation) = left_to_parse.strip_prefix(')') {
            // When we get done with a variation we need to pop off the stack
            if let Some(popped) = self.stack.pop() {
                if after_end_of_variation.is_empty() {
                    // if nothing else left to parse then end parser
                    return Ok((after_end_of_variation, after_end_of_variation));
                }

                if self
                    .get_variation_creator()
                    .unwrap_or(&self.graph.get_tree_root())
                    == &popped
                {
                    let children = self.graph.get_children(popped);
                    self.prev_node = children
                        .into_iter()
                        .nth(self.stack.get_variation_count(popped))
                        .unwrap();
                } else {
                    self.prev_node = popped;
                }
                left_to_parse = after_end_of_variation.trim_start();
            } else {
                // If we expect to pop an item but is already empty
                // There were more opening parenthes than closed parentheses
                panic!("Parentheses closed improperly, {after_end_of_variation}")
            }

            // Run the rest of the input
            return self.move_entry(left_to_parse);
        }

        let (rest, _) = move_number.opt().parse(left_to_parse)?;
        // TODO: This doesn't take into account promotion
        let (rest, move_text) = move_text.parse(rest.trim_start())?;
        let (rest, nag) = nag.opt().parse(rest.trim_start())?;
        let (rest, comment) = comment.opt().parse(rest.trim_start())?;
        let parsed_move: Move = (
            move_text,
            comment.map(std::string::ToString::to_string),
            nag,
        );

        self.add_node(
            parsed_move,
            match parent_key {
                Some(k) => Some(k),
                None => Some(self.prev_node),
            },
        );
        Ok((rest.trim_start(), rest.trim_start()))
    }

    fn generate_next_fen(current_fen: &Fen, move_text: MoveText) -> Fen {
        let mut board = Board::from_str(current_fen).expect("Should be valid FEN");
        if move_text == "O-O" {
            match board.side_to_move() {
                Color::White => board
                    .update(crate::common::r#move::Move {
                        from: Square::E1,
                        to: Square::G1,
                    })
                    .to_string(),
                Color::Black => board
                    .update(crate::common::r#move::Move {
                        from: Square::E8,
                        to: Square::G8,
                    })
                    .to_string(),
            }
        } else if move_text == "O-O-O" {
            match board.side_to_move() {
                Color::White => board
                    .update(crate::common::r#move::Move {
                        from: Square::E1,
                        to: Square::C1,
                    })
                    .to_string(),
                Color::Black => board
                    .update(crate::common::r#move::Move {
                        from: Square::E8,
                        to: Square::C8,
                    })
                    .to_string(),
            }
        } else {
            let piece =
                Piece::try_from(&move_text.chars().next().unwrap()).expect("Should be valid piece");
            // TODO: This doesn't take into account promotion
            let dest = Square::from_str(&move_text[move_text.len() - 2..])
                .expect("Should be valid square");

            let src = board.get_valid_moves_to(dest, piece);
            assert!(!src.is_empty());
            if src.len() == 1 {
                board.update(crate::common::r#move::Move {
                    from: src.into_iter().next().unwrap(),
                    to: dest,
                });
            } else {
                // Nexd5 vs Ned5
                let disambiguation = match piece {
                    Piece::Pawn => {
                        File::from_str(&move_text.chars().next().unwrap().to_string()).unwrap()
                    }
                    _ => File::from_str(&move_text.chars().nth(1).unwrap().to_string()).unwrap(),
                };
                for s in src {
                    if s.file() == disambiguation {
                        board
                            .update(crate::common::r#move::Move { from: s, to: dest })
                            .to_string();
                    }
                }
            }
            board.to_string()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_move_entries() {
        let mut parser = Parser::new();
        let _ = parser.move_entry("1.d4 e5").unwrap();

        assert_eq!(parser.graph.0.count(), 2);
    }

    #[test]
    fn it_move_entries_without_move_number() {
        let mut parser = Parser::new();
        let _ = parser.move_entry("d4?").unwrap();
        assert_eq!(parser.graph.0.count(), 2);
    }

    #[test]
    fn parses_nested_variations() {
        let parser = Parser::new();
        let res = parser
            .parse("1. d4 ( 1. e4 e5 (2... Nf6 3. Nh3) ) d5 2.Nf3 (2. a4) 1-0")
            // .parse("1. d4 ( 1. e4 e5 2. Nf3 Nf6 (2... a6 (2... b6 )(2... Nc6) 3. Na3)  ) d5 1-0")
            // .parse("1.d4 e5 (1...e6 2.e4)")
            .unwrap();

        assert_eq!(res.0.count(), 9);
    }

    #[test]
    #[should_panic]
    fn should_panic_for_invalid_pgn() {
        let pgn = Parser::new();
        let _ = pgn.parse("1.z@2").unwrap();
    }

    #[test]
    fn parses_to_linear_graph() {
        let pgn = Parser::new();
        let res = pgn.parse("1.e4 e5 2.d4 d5 1-0 ").unwrap();
        assert_eq!(res.0.count(), 5);
    }

    #[test]
    fn game_with_nested_comment_and_variations() {
        let pgn = Parser::new();
        let res = pgn
            .parse("1.e4 {This is a comment}(1. d4 Nf6 ) e5 1-0")
            .unwrap();

        assert_eq!(res.0.count(), 5);
    }

    #[test]
    fn debug() {
        let pgn = Parser::new();
        let res = pgn
            .parse(
                "1. e4 e5 2. Nf3 Nc6 3. Bb5 {This opening is called the Ruy Lopez.} 3... a6
                     4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. c3 O-O 9. h3 Nb8 10. d4 Nbd7
    11. c4 c6 12. cxb5 axb5 13. Nc3 Bb7 14. Bg5 b4 15. Nb1 h6 16. Bh4 c5 17. dxe5
                Nxe4 18. Bxe7 Qxe7 19. exd6 Qf6 20. Nbd2 Nxd6 21. Nc4 Nxc4 22. Bxc4 Nb6
                23. Ne5 Rae8 24. Bxf7+ Rxf7 25. Nxf7 Rxe1+ 26. Qxe1 Kxf7 27. Qe3 Qg5 28. Qxg5
                hxg5 29. b3 Ke6 30. a3 Kd6 31. axb4 cxb4 32. Ra5 Nd5 33. f3 Bc8 34. Kf2 Bf5
                35. Ra7 g6 36. Ra6+ Kc5 37. Ke1 Nf4 38. g3 Nxh3 39. Kd2 Kb5 40. Rd6 Kc5 41. Ra6
                Nf2 42. g4 Bd3 43. Re6
        1/2-1/2",
            )
            .unwrap();
        assert_eq!(res.0.count(), 5);
    }

    #[test]
    fn next_fen() {
        let res =
            Parser::generate_next_fen(&STARTING_POSITION_FEN.to_string(), &String::from("d4"));
        assert_eq!(
            res,
            "rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq - 0 1"
        );
    }
    // #[test]
    // fn debug_fen() {
    //     let res = Parser::generate_next_fen(
    //         &String::from("r1bq1rk1/3nbppp/p1pp1n2/1P2p3/3PP3/1B3N1P/PP3PP1/RNBQR1K1 b - - 0 12"),
    //         &String::from("axb5"),
    //     );
    // }
}

// impl<'a> From<AdjacencyList<'a>> for MoveTree {
//     fn from(graph: AdjacencyList<'a>) -> Self {
//         let mut tree = MoveTree::new();
//         for (key, (entry, edges)) in &graph.0 {
//             if let ListEntry::Node((move_text, _, _), fen, parent) = entry {
//                 let node = TreeNode {
//                     notation: (*move_text).to_string(),
//                     fen: fen.to_string(),
//                 };
//                 tree.0.new_node(node);
//                 for edge in
//             };
//         }
//         // for edge in par
//         tree
//     }
// }
