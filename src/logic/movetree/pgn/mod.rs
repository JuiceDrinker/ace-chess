#![allow(dead_code)]

use std::collections::{HashMap, HashSet, VecDeque};

mod parsers;
use nom::{IResult, Parser};
use nom_supreme::{error::ErrorTree, ParserExt};

use crate::{error::Error, prelude::Result};

use self::parsers::{comment, move_number, move_text};

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
enum Edge {
    Variation(Key),
    Stem(Key),
}
type Key = usize;
type MoveText<'a> = &'a str;
type Edges = HashSet<Edge>;
type MoveTextAndComment<'a> = (MoveText<'a>, Option<Comment<'a>>);
type AdjacencyList<'a> = HashMap<Key, (MoveTextAndComment<'a>, Edges)>;
type Comment<'a> = &'a str;

#[derive(Default, Debug, Clone)]
struct VariationStack(VecDeque<Key>);

impl VariationStack {
    fn new() -> Self {
        VariationStack::default()
    }

    pub fn is_variation_stack_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn pop(&mut self) -> Option<Key> {
        self.0.pop_back()
    }

    pub fn push(&mut self, key: Key) {
        self.0.push_back(key)
    }

    pub fn peek(&self) -> Option<&Key> {
        self.0.back()
    }
}

#[derive(Default, Debug, Clone)]
struct PgnParser<'a> {
    stack: VariationStack,
    graph: AdjacencyList<'a>,
    current_key: usize,
    prev_node: Option<Key>,
}

impl<'a> PgnParser<'a> {
    fn new() -> Self {
        PgnParser::default()
    }

    fn add_node(&mut self, move_text: MoveText<'a>, comment: Option<Comment<'a>>) {
        self.current_key += 1;
        self.prev_node = Some(self.current_key);
        self.graph
            .insert(self.current_key, ((move_text, comment), HashSet::new()));
    }

    fn parse(mut self, input: &'a str) -> Result<AdjacencyList> {
        let mut left_to_parse = input;
        while !left_to_parse.trim_start().is_empty() {
            // TODO: Make return type nicer since we never use second arg
            if let Ok((rest, _)) = self.move_entry(left_to_parse.trim_start()) {
                left_to_parse = rest;
            } else {
                return Err(Error::ParseError);
            }
        }
        Ok(self.graph)
    }

    fn prev_node(&self) -> Option<&Key> {
        self.prev_node.as_ref()
    }

    fn get_variation_creator(&self) -> Option<&Key> {
        self.stack.peek()
    }

    // Base case
    fn move_entry(&mut self, input: &'a str) -> IResult<&'a str, &'a str, ErrorTree<&'a str>> {
        if input.trim_start().starts_with("1-0")
            || input.trim_start().starts_with("1-0")
            || input.trim_start().starts_with("1/2/-1/2")
        {
            // Eats rest of input
            // TODO: Keep on processing file if there are more games
            return Ok(("", ""));
        }
        let mut left_to_parse = input.trim_start();
        let prev_node = &self.prev_node();
        if let Some(after_start_of_variation) = left_to_parse.strip_prefix('(') {
            // If we enter a variation the last node we saw was the one that got us here
            self.stack
                .push(*prev_node.expect("Variation cannot be root"));
            left_to_parse = after_start_of_variation;
        } else if let Some(after_end_of_variation) = left_to_parse.trim_start().strip_prefix(')') {
            // When we get done with a variation we need to pop off the stack
            if let Some(popped) = self.stack.pop() {
                if after_end_of_variation.is_empty() {
                    // if nothing else left to parse then end parser
                    return Ok((after_end_of_variation, after_end_of_variation));
                } else {
                    // The popped last variation stack item is the previous node
                    // who's variation we just exited
                    left_to_parse = after_end_of_variation.trim_start();
                    self.prev_node = Some(popped);
                }
            } else {
                // If we expect to pop an item but is already empty
                // There were more opening parenthes than closed parentheses
                panic!("Parentheses closed improperly, {after_end_of_variation}")
            }

            // Run the rest of the input
            // Need to call
            return self.move_entry(left_to_parse);
        }

        let (rest, _) = move_number.opt().parse(left_to_parse.trim_start())?;
        let (rest, move_text) = move_text.parse(rest.trim_start())?;
        let (rest, comment) = comment.opt().parse(rest.trim_start())?;

        match (self.get_variation_creator(), self.prev_node) {
            // Root of tree just add_node
            (None, None) => self.add_node(move_text, comment),

            // Middle of a stem
            (None, Some(prev_node_key)) => {
                self.add_node(move_text, comment);
                self.graph.entry(prev_node_key).and_modify(|(_, keys)| {
                    keys.insert(Edge::Stem(self.current_key));
                });
            }

            // We are in midst of a variation
            (Some(variation_creator_key), Some(prev_node_key)) => {
                // First move of variation
                if *variation_creator_key == prev_node_key {
                    self.graph
                        .entry(*variation_creator_key)
                        .and_modify(|(_, keys)| {
                            keys.insert(Edge::Variation(self.current_key + 1));
                        });
                } else {
                    // Or continuing variation
                    self.graph.entry(prev_node_key).and_modify(|(_, keys)| {
                        keys.insert(Edge::Stem(self.current_key + 1));
                    });
                }
                self.add_node(move_text, comment);
            }
            // Invalid state
            (Some(_), None) => panic!("Can't have no root but variation creator"),
        }

        Ok((rest.trim_start(), rest.trim_start()))
    }
}

mod test {
    use super::*;

    #[test]
    fn it_move_entries() {
        let mut parser = PgnParser::new();
        let _ = parser.move_entry("1.d4 e5").unwrap();
        assert_eq!(parser.graph[&1_usize], (("d4", None), HashSet::new()));
    }

    #[test]
    fn it_move_entries_without_move_number() {
        let mut parser = PgnParser::new();
        let _ = parser.move_entry("d4").unwrap();
        assert_eq!(parser.graph[&1_usize], (("d4", None), HashSet::new()));
    }

    #[test]
    fn parses_nested_variations() {
        let parser = PgnParser::new();
        let res = parser
            .parse("1. d4 ( 1. e4 e5 (2... Nf6) ) d5 1-0")
            .unwrap();

        assert_eq!(res.len(), 5)
    }

    #[test]
    #[should_panic]
    fn should_panic_for_invalid_pgn() {
        let pgn = PgnParser::new();
        let _ = pgn.parse("1.z@2").unwrap();
    }

    #[test]
    fn parses_to_linear_graph() {
        let pgn = PgnParser::new();
        let res = pgn.parse("1.e4 e5 2.d4 d5 1-0 ").unwrap();
        assert_eq!(res.len(), 4);
    }

    #[test]
    fn game_with_nested_comment_and_variations() {
        let pgn = PgnParser::new();
        let res = pgn
            .parse("1.e4 {This is a comment}(1. d4 Nf6 ) e5 1-0")
            .unwrap();

        assert_eq!(res.len(), 4);
    }
}
