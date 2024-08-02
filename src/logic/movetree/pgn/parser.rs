use crate::error::{Error, ParseKind};
use std::str::FromStr;

use crate::logic::movetree::treenode::TreeNode;

struct PgnParser {
    cursor: usize,
    characters: Vec<char>,
}

impl PgnParser {
    pub fn parse(input: &str) -> Result<indextree::Arena<TreeNode>, Error> {
        todo!()
    }
}
// Grammar
// R: 1 … 8               # Rank
// F: a … h               # File
// P: N, B, R, Q, K       # Piece
// PM: PFR | P'x'FR | PFxFR   # Piece Move
// PM1: FR | FxFR | FR=P | FxFR=P  # Pawn Move
// C: { string }          # Comment
// C1: '0-0' | '0-0-0'    # Castling
// MN: [0-9]+             # Move Number
// D: .                   # Dot
// CH: + | #              # Check/Checkmate
// M: (PM | PM1 | C1) CH?  # Move (with optional check/checkmate)
// MT: M | MN D M | MN DDD M  # Move Text
// V: ( E )               # Variation
// E: MT | C | V | E E    # Element (allows for comments and variations between moves)
// GT: '1-0' | '0-1' | '1/2-1/2' | '*'  # Game Termination
// TS: '[' string string ']'  # Tag Section
// G: TS* E* GT           # Game (with optional tags, multiple elements, and termination)

#[derive(Debug, PartialEq)]
pub enum Nag {
    Good,
    Excellent,
    Interesting,
    Blunder,
    Poor,
    Dubious,
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

// #[cfg(test)]
// mod test {
//     use super::*;
//     use pretty_assertions::assert_eq;
//
//     #[test]
//     fn it_move_numbers() {
//         let (_, parsed) = move_number("1.").unwrap();
//         assert_eq!(parsed, "1.");
//     }
//
//     #[test]
//     fn it_move_numbers_black() {
//         let (_, parsed) = move_number("1...").unwrap();
//         assert_eq!(parsed, "1...");
//     }
//
//     #[test]
//     #[should_panic]
//     fn panics_on_rank_outside_bounds() {
//         rank("9").unwrap();
//     }
//
//     #[test]
//     fn parses_rank() {
//         let (_, move_text) = rank("7").unwrap();
//         assert_eq!(move_text, "7");
//     }
//
//     #[test]
//     fn parses_file() {
//         let (_, move_text) = file("b").unwrap();
//         assert_eq!(move_text, "b");
//     }
//
//     #[test]
//     #[should_panic]
//     fn panics_on_invalid_file() {
//         file("y").unwrap();
//     }
//
//     #[test]
//     fn parses_move_text() {
//         let (_, move_text) = move_text("Nd5").unwrap();
//         assert_eq!(move_text, "Nd5");
//     }
//
//     #[test]
//     fn parses_move_text_with_disambiguation() {
//         let (_, move_text) = move_text("Nbd5 Nd2").unwrap();
//         assert_eq!(move_text, "Nbd5");
//     }
//
//     #[test]
//     fn parses_move_text_with_capture() {
//         let (_, move_text) = move_text("Nxd5").unwrap();
//         assert_eq!(move_text, "Nxd5");
//     }
//
//     #[test]
//     fn parses_capture() {
//         let (_, move_text) = capture("xe5").unwrap();
//         assert_eq!(move_text, "x");
//     }
//
//     #[test]
//     fn parses_move_text_with_disambiguated_capture() {
//         let (_, move_text) = move_text("Nexd5").unwrap();
//         assert_eq!(move_text, "Nexd5");
//     }
//
//     #[test]
//     fn parses_disambiguated_capture() {
//         let (_, move_text) = disambiguated_capture("ex").unwrap();
//         assert_eq!(move_text, "ex");
//     }
//
//     #[test]
//     fn parses_nag_interesting() {
//         let (_, nag) = nag("!?").unwrap();
//         assert_eq!(nag, Nag::Interesting);
//     }
//
//     #[test]
//     fn parses_nag_dubious() {
//         let (_, nag) = nag("?!").unwrap();
//         assert_eq!(nag, Nag::Dubious);
//     }
//
//     #[test]
//     fn parses_nag_poor() {
//         let (_, nag) = nag("?").unwrap();
//         assert_eq!(nag, Nag::Poor);
//     }
//
//     #[test]
//     fn parses_nag_blunder() {
//         let (_, nag) = nag("??").unwrap();
//         assert_eq!(nag, Nag::Blunder);
//     }
//
//     #[test]
//     fn parses_nag_good() {
//         let (_, nag) = nag("!").unwrap();
//         assert_eq!(nag, Nag::Good);
//     }
//
//     #[test]
//     fn parses_nag_excellent() {
//         let (_, nag) = nag("!!").unwrap();
//         assert_eq!(nag, Nag::Excellent);
//     }
//
//     #[test]
//     fn parses_comments() {
//         let (_, comment) = comment("{This is a comment}").unwrap();
//         assert_eq!(comment, "This is a comment");
//     }
// }
