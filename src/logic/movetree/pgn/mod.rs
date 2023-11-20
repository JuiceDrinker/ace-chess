#![allow(dead_code)]

use std::collections::{HashMap, HashSet, VecDeque};

mod parsers;
use nom::{IResult, Parser};
use nom_supreme::{error::ErrorTree, ParserExt};

use crate::prelude::Result;

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
struct PgnParser<'a> {
    variation_stack: VecDeque<Key>,
    graph: AdjacencyList<'a>,
    current_key: usize,
    prev_node: Option<Key>,
}

impl<'a> PgnParser<'a> {
    fn new() -> Self {
        PgnParser::default()
    }

    fn is_variation_stack_empty(&self) -> bool {
        self.variation_stack.is_empty()
    }

    fn pop(&mut self) -> Option<Key> {
        self.variation_stack.pop_back()
    }

    fn push_variation_stack(&mut self, key: Key) {
        self.variation_stack.push_back(key)
    }

    fn add_node(&mut self, move_text: MoveText<'a>, comment: Option<Comment<'a>>) {
        self.current_key += 1;
        self.prev_node = Some(self.current_key);
        self.graph
            .insert(self.current_key, ((move_text, comment), HashSet::new()));
    }

    fn parse(self, input: &'a str) -> Result<Self> {
        let mut left_to_parse = input;
        let mut original_parser = PgnParser::new();
        // 1.e4 {This is a comment}(1. d4 e5 ) e5 1-0
        loop {
            if !left_to_parse.trim_start().is_empty() {
                if left_to_parse.trim_start().starts_with("1-0")
                    || left_to_parse.trim_start().starts_with("0-1")
                {
                    return Ok(original_parser);
                }
                if let Ok((rest, parser)) =
                    PgnParser::move_entry(left_to_parse.trim_start(), &mut original_parser)
                {
                    original_parser = parser;
                    left_to_parse = rest;
                }
            } else {
                return Ok(original_parser);
            }
        }
    }

    fn prev_node(&self) -> Option<&Key> {
        self.prev_node.as_ref()
    }

    fn get_variation_creator(&self) -> Option<&Key> {
        // peek
        self.variation_stack.back()
    }

    // Base case
    fn move_entry(
        input: &'a str,
        parser: &mut PgnParser<'a>,
    ) -> IResult<&'a str, PgnParser<'a>, ErrorTree<&'a str>> {
        if input.trim_start().starts_with("1-0") || input.trim_start().starts_with("0-1") {
            return Ok(("", parser.to_owned()));
        }
        let mut new_parser = parser.clone();
        let mut left_to_parse = input;
        let prev_node = &parser.prev_node();
        if let Some(after_start_of_variation) = left_to_parse.trim_start().strip_prefix('(') {
            // If we enter a variation the last node we saw was the one that got us here
            // dbg!(*new_parser.unwrap());
            new_parser.push_variation_stack(*prev_node.expect("Variation cannot be root"));
            left_to_parse = after_start_of_variation;
        } else if let Some(after_end_of_variation) = left_to_parse.trim_start().strip_prefix(')') {
            // When we get done with a variation we need to pop off the stack
            let popped = new_parser.pop();
            // If we pop the last variation stack item, set the previous item to prev_node
            // if new_parser.is_variation_stack_empty() {
            new_parser.prev_node = popped;
            // }

            // left_to_parse = after_end_of_variation;
            // Run the rest of the input
            return PgnParser::move_entry(after_end_of_variation, &mut new_parser);
        }
        let (rest, _) = move_number.opt().parse(left_to_parse.trim_start())?;
        let (rest, move_text) = move_text.parse(rest.trim_start())?;
        let (rest, comment) = comment.opt().parse(rest.trim_start())?;

        // if !new_parser.is_variation_stack_empty()
        match (new_parser.get_variation_creator(), parser.prev_node()) {
            // Root of tree just add_node
            (None, None) => new_parser.add_node(move_text, comment),

            // Was a previous node but no variation
            // So just add and update prev_node to point to this one
            (None, Some(prev_node_key)) => {
                new_parser.add_node(move_text, comment);
                new_parser
                    .graph
                    .entry(*prev_node_key)
                    .and_modify(|(_, keys)| {
                        keys.insert(Edge::Stem(new_parser.current_key));
                    });
            }
            (Some(_), None) => panic!("Can't have no root but variation creator"),
            // We are in midst of a variation
            (Some(variation_creator_key), Some(prev_node_key)) => {
                // First move of variation
                if variation_creator_key == prev_node_key {
                    new_parser
                        .graph
                        .entry(*variation_creator_key)
                        .and_modify(|(_, keys)| {
                            keys.insert(Edge::Variation(new_parser.current_key + 1));
                        });
                } else {
                    // Or continuing variation
                    new_parser
                        .graph
                        .entry(*prev_node_key)
                        .and_modify(|(_, keys)| {
                            keys.insert(Edge::Stem(new_parser.current_key + 1));
                        });
                }
                new_parser.add_node(move_text, comment);
            }
        }

        // dbg!(&parser);
        Ok((rest.trim_start(), new_parser.to_owned()))
    }
}

mod test {
    use crate::logic::movetree::pgn::PgnParser;

    #[test]
    fn parses_nested_variations() {
        let parser = PgnParser::new();
        let parsed = parser.parse("1. d4 (1. e4 e5 (2... Nf6) ) d5 1-0").unwrap();

        dbg!(parsed);
        // assert_eq!(
        //     parsed,
        //     vec![
        //         ParsedMove {
        //             r#move: "e5",
        //             comment: None,
        //             variations: None
        //         },
        //         ParsedMove {
        //             r#move: "d4",
        //             comment: None,
        //             variations: Some(vec![ParsedMove {
        //                 r#move: "Nf3",
        //                 comment: None,
        //                 variations: None
        //             }])
        //         }
        //     ]
        // )
    }

    // #[test]
    // #[should_panic]
    // fn should_panic_for_invalid_pgn() {
    //     let res = pgn("1.z@2").unwrap();
    //     dbg!(&res);
    // }

    // #[test]
    fn parses_first_move() {
        let pgn = PgnParser::new();
        let res = pgn.parse("1.e4 e5 2.d4 d5 1-0 ").unwrap();
        dbg!(res);
        // assert_eq!(res.len(), 4);
        // assert_eq!(
        //     res[0],
        //     ParsedMove {
        //         r#move: "e4",
        //         comment: None,
        //         variations: vec![],
        //     }
        // );
        // assert_eq!(
        //     res[1],
        //     ParsedMove {
        //         r#move: "e5",
        //         comment: None,
        //         variations: vec![],
        //     }
        // );
        // assert_eq!(
        //     res[2],
        //     ParsedMove {
        //         r#move: "d4",
        //         comment: None,
        //         variations: vec![],
        //     }
        // );
        // assert_eq!(
        //     res[3],
        //     ParsedMove {
        //         r#move: "d5",
        //         comment: None,
        //         variations: vec![],
        //     }
        // );
    }

    // #[test]
    fn game_with_nested_comment_and_variations() {
        let pgn = PgnParser::new();
        let res = pgn
            .parse("1.e4 {This is a comment}(1. d4 Nf6 ) e5 1-0")
            .unwrap();
        dbg!(res);
    }

    // #[test]
    // fn game_with_comment_and_variations() {
    //     let (_, parsed) = pgn("1.e4 {This is a comment}(1... e5 2.d4 )").unwrap();
    //     dbg!(&parsed);
    //     assert_eq!(parsed.len(), 1);
    //     assert_eq!(
    //         parsed,
    //         vec![ParsedMove {
    //             r#move: "e4",
    //             comment: Some("This is a comment"),
    //             variations: vec![
    //                 ParsedMove {
    //                     r#move: "e5",
    //                     comment: None,
    //                     variations: vec![]
    //                 },
    //                 ParsedMove {
    //                     r#move: "d4",
    //                     comment: None,
    //                     variations: vec![]
    //                 }
    //             ]
    //         }]
    //     );
    // }
}
