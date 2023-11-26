#![allow(dead_code)]
#![warn(clippy::all)]
const STARTING_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    str::FromStr,
};

mod parsers;
use nom::{IResult, Parser};
use nom_supreme::{error::ErrorTree, ParserExt};

use crate::{
    error::{Error, ParseKind},
    prelude::Result,
};

use crate::logic::movetree::pgn::parsers::{comment, move_number, move_text, nag};

use super::treenode::Fen;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
enum Edge {
    Variation(Key),
    Stem(Key),
}

impl Ord for Edge {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let a = self.get_key();
        let b = other.get_key();
        a.cmp(b)
    }
}

impl PartialOrd for Edge {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Edge {
    const fn get_key(&self) -> &Key {
        match self {
            Self::Variation(k) | Self::Stem(k) => k,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
enum ListEntry<'a> {
    Root(Fen),
    Node(Move<'a>),
}
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

type Key = usize;
type MoveText<'a> = &'a str;
type Edges = HashSet<Edge>;
type Move<'a> = (MoveText<'a>, Option<Comment<'a>>, Option<Nag>);

#[derive(Debug, Clone, PartialEq, Default)]
struct AdjacencyList<'a>(BTreeMap<Key, (ListEntry<'a>, Edges)>);
impl<'a> AdjacencyList<'a> {
    fn get_parent(&self, key: Key, ancestor: Key) -> Option<Key> {
        if let Some((_, edges)) = self.0.get(&ancestor) {
            if edges.is_empty() {
                return None;
            }
            if edges.contains(&Edge::Variation(key)) || edges.contains(&Edge::Stem(key)) {
                Some(ancestor)
            } else {
                edges
                    .iter()
                    .fold(Vec::new(), |mut acc, edge| {
                        acc.push(self.get_parent(key, *edge.get_key()));
                        acc
                    })
                    .into_iter()
                    .flatten()
                    .next()
            }
        } else {
            None
        }
    }
}
type Comment<'a> = &'a str;

#[derive(Default, Debug, Clone)]
struct Variation(VecDeque<Key>, HashMap<Key, usize>);

impl Variation {
    fn new() -> Self {
        Self::default()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn get_variation_count(&self, key: Key) -> usize {
        *self.1.get(&key).unwrap_or(&0)
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn pop(&mut self) -> Option<Key> {
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

    fn push(&mut self, key: Key) {
        self.0.push_back(key);
        self.1
            .entry(key)
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    fn peek(&self) -> Option<&Key> {
        self.0.back()
    }
}

#[derive(Default, Debug, Clone)]
struct PgnParser<'a> {
    stack: Variation,
    graph: AdjacencyList<'a>,
    current_key: usize,
    prev_node: Key,
}

impl<'a> PgnParser<'a> {
    fn new() -> Self {
        Self {
            stack: Variation::default(),
            graph: {
                let mut graph = BTreeMap::new();
                graph.insert(
                    0,
                    (
                        ListEntry::Root(STARTING_POSITION_FEN.to_owned()),
                        HashSet::new(),
                    ),
                );
                AdjacencyList(graph)
            },
            current_key: 0,
            prev_node: 0,
        }
    }

    fn add_node(&mut self, r#move: Move<'a>) {
        self.current_key += 1;
        self.prev_node = self.current_key;
        self.graph
            .0
            .insert(self.current_key, (ListEntry::Node(r#move), HashSet::new()));
    }

    fn parse(mut self, input: &'a str) -> Result<AdjacencyList> {
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

    const fn prev_node(&self) -> &Key {
        &self.prev_node
    }

    fn get_variation_creator(&self) -> Option<&Key> {
        self.stack.peek()
    }

    // Base case
    fn move_entry(&mut self, input: &'a str) -> IResult<&'a str, &'a str, ErrorTree<&'a str>> {
        let mut left_to_parse = input.trim_start();
        if left_to_parse.starts_with("1-0")
            || input.trim_start().starts_with("1-0")
            || input.trim_start().starts_with("1/2/-1/2")
        {
            // Eats rest of input
            // TODO: Keep on processing file if there are more games
            return Ok(("", ""));
        }
        if let Some(after_start_of_variation) = left_to_parse.strip_prefix('(') {
            if self.stack.is_empty() {
                // We are in first move of variation
                let parent = self.graph.get_parent(self.prev_node, 0).unwrap();
                self.graph.0.entry(parent).and_modify(|(_, edges)| {
                    edges.insert(Edge::Variation(self.current_key + 1));
                });
                self.stack.push(parent);
            } else {
                let parent = self
                    .graph
                    .get_parent(
                        self.prev_node,
                        *self.get_variation_creator().expect("Stack is not empty"),
                    )
                    .expect("Previous node always exists");
                self.stack.push(parent);
                self.graph.0.entry(parent).and_modify(|(_, edges)| {
                    edges.insert(Edge::Variation(self.current_key + 1));
                });
            }
            // If we enter a variation the last node we saw was the one that got us here
            // First move of a new variation
            left_to_parse = after_start_of_variation;
        } else if let Some(after_end_of_variation) = left_to_parse.strip_prefix(')') {
            // When we get done with a variation we need to pop off the stack
            if let Some(popped) = self.stack.pop() {
                if after_end_of_variation.is_empty() {
                    // if nothing else left to parse then end parser
                    return Ok((after_end_of_variation, after_end_of_variation));
                }
                if self.stack.peek().unwrap_or(&0) == &popped {
                    let (_, edges) = self.graph.0.get(&popped).unwrap();
                    let mut sorted_edges = edges.iter().collect::<Vec<_>>();
                    sorted_edges.sort();

                    self.prev_node = *sorted_edges
                        .into_iter()
                        .filter(|e| *e.get_key() != self.current_key)
                        .nth(
                            self.stack
                                .get_variation_count(*self.get_variation_creator().unwrap_or(&0)),
                        )
                        .unwrap()
                        .get_key();
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
            // Need to call
            return self.move_entry(left_to_parse);
        } else {
            self.graph.0.entry(self.prev_node).and_modify(|(_, edges)| {
                edges.insert(Edge::Stem(self.current_key + 1));
            });
        }

        let (rest, _) = move_number.opt().parse(left_to_parse)?;
        let (rest, move_text) = move_text.parse(rest.trim_start())?;
        let (rest, nag) = nag.opt().parse(rest.trim_start())?;
        let (rest, comment) = comment.opt().parse(rest.trim_start())?;
        let parsed_move: Move = (move_text, comment, nag);

        self.add_node(parsed_move);
        Ok((rest.trim_start(), rest.trim_start()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_move_entries() {
        let mut parser = PgnParser::new();
        let _ = parser.move_entry("1.d4 e5").unwrap();
        assert_eq!(
            parser.graph.0[&1_usize],
            (ListEntry::Node(("d4", None, None)), HashSet::new())
        );
    }

    #[test]
    fn it_move_entries_without_move_number() {
        let mut parser = PgnParser::new();
        let _ = parser.move_entry("d4?").unwrap();
        assert_eq!(
            parser.graph.0[&1_usize],
            (
                ListEntry::Node(("d4", None, Some(Nag::Poor))),
                HashSet::new()
            )
        );
    }

    #[test]
    fn parses_nested_variations() {
        let parser = PgnParser::new();
        let res = parser
            .parse("1. d4 ( 1. e4 e5 (2... Nf6 3. Nh3) ) d5 2.Nf3 (2. a4) 1-0")
            // .parse("1. d4 ( 1. e4 e5 2. Nf3 Nf6 (2... a6 (2... b6 )(2... Nc6) 3. Na3)  ) d5 1-0")
            // .parse("1.d4 e5 (1...e6 2.e4)")
            .unwrap();

        assert_eq!(dbg!(res.0).len(), 9);
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
        assert_eq!(res.0.len(), 5);
    }

    #[test]
    fn game_with_nested_comment_and_variations() {
        let pgn = PgnParser::new();
        let res = pgn
            .parse("1.e4 {This is a comment}(1. d4 Nf6 ) e5 1-0")
            .unwrap();

        assert_eq!(res.0.len(), 5);
    }
}
