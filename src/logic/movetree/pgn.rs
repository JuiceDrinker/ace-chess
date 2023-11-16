#![allow(dead_code)]
use std::{marker::PhantomData, num::ParseIntError};

use nom::{
    branch::alt,
    bytes::complete::{take_until, take_while_m_n},
    character::complete::char,
    character::complete::{multispace0, multispace1, space0, u8},
    combinator::{map, opt, recognize},
    error::{FromExternalError, ParseError},
    multi::{fold_many1, many0, many1, separated_list0, separated_list1},
    sequence::{delimited, tuple, Tuple},
    Err, IResult, Parser,
};
use nom_supreme::{
    error::{BaseErrorKind, ErrorTree, Expectation, GenericErrorTree},
    tag::complete::tag,
    ParserExt,
};

#[derive(Debug, PartialEq, Clone)]
pub struct ParsedMove<'a> {
    r#move: &'a str,
    variations: Vec<Vec<ParsedMove<'a>>>,
    comment: Option<&'a str>,
}

impl<'a> ParsedMove<'a> {
    fn new(
        r#move: &'a str,
        comment: Option<&'a str>,
        variations: Option<Vec<ParsedMove<'a>>>,
    ) -> ParsedMove<'a> {
        ParsedMove {
            r#move,
            variations: variations.unwrap_or_default(),
            comment,
        }
    }
}

fn parse_rank(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    take_while_m_n(1, 1, |c: char| matches!(c, '1'..='8')).parse(input)
}

fn parse_file(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    take_while_m_n(1, 1, |c: char| matches!(c, 'a'..='h')).parse(input)
}

fn parse_piece(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    take_while_m_n(1, 1, |c: char| matches!(c, 'N' | 'B' | 'R' | 'Q' | 'K')).parse(input)
}

fn parse_promotion_piece(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    take_while_m_n(1, 1, |c: char| matches!(c, 'N' | 'B' | 'R' | 'Q')).parse(input)
}

fn parse_disambiguated_capture(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    recognize(tuple((parse_file, tag("x")))).parse(input)
}

fn parse_capture(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    alt((tag("x"), parse_disambiguated_capture)).parse(input)
}

fn parse_full_capture(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    recognize(tuple((parse_piece, parse_capture, parse_file, parse_rank))).parse(input)
}

fn parse_basic(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    recognize(tuple((parse_piece, parse_file, parse_rank))).parse(input)
}

fn parse_disambugated_move(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    recognize(tuple((parse_piece, parse_file, parse_file, parse_rank))).parse(input)
}

fn basic_pawn_move(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    recognize(tuple((parse_file, parse_rank)))
        .context("basic pawn move")
        .parse(input)
}

fn pawn_capture(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    recognize(tuple((parse_file, tag("x"), parse_rank)))
        .context("pawn capture")
        .parse(input)
}

fn pawn_promotion(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    recognize(tuple((
        parse_file,
        opt(tag("x")),
        parse_rank,
        tag("="),
        parse_promotion_piece,
    )))
    .parse(input)
}

fn pawn_move(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    alt((basic_pawn_move, pawn_capture, pawn_capture)).parse(input)
}

fn move_text(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    let (rest, move_text) =
        alt((pawn_move, parse_piece_move, tag("0-0"), tag("0-0-0"))).parse(input)?;
    Ok((rest, move_text))
}

fn parse_piece_move(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    // Nd5, Nbd5, Nbxd5, Nxd5
    parse_basic
        .or(parse_disambugated_move)
        .or(parse_full_capture)
        .parse(input)
}

fn comment(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    let (rest, comment) =
        ws(delimited(tag("{"), take_until("}"), tag("}"))).parse(input.trim_start())?;
    // dbg!((rest, comment));
    Ok((rest, comment))
}

fn ws<'a, O, E: ParseError<&'a str>, F: Parser<&'a str, O, E>>(f: F) -> impl Parser<&'a str, O, E> {
    delimited(multispace0, f, multispace0)
}

fn move_entry(input: &str) -> IResult<&str, ParsedMove, ErrorTree<&str>> {
    // dbg!(input);
    let (rest, _) = move_number.opt().parse(input)?;
    let (rest, mut move_text) =
        map(move_text, |text| ParsedMove::new(text, None, None)).parse(rest)?;
    dbg!(&move_text);
    let (rest, comment) = comment.opt().parse(rest)?;
    move_text.comment = comment;
    let (rest, variations) = variation.opt().parse(rest)?;

    dbg!((move_text.clone(), &variations));
    let _ = variations
        .into_iter()
        .map(|mut variation| move_text.clone().variations.append(&mut variation));

    Ok((rest, move_text))
}

fn fold_move_entry(input: &str) -> IResult<&str, Vec<ParsedMove>, ErrorTree<&str>> {
    // dbg!(input);
    // let (rest, nested_variation) = variation.opt().parse(input)?;
    //
    many1(map(ws(move_entry), |entry| dbg!(entry))).parse(input)

    // let variation: Vec<ParsedMove> = outer_variation
    //     .unwrap_or_default()
    //     .into_iter()
    //     // .chain(nested_variation.unwrap_or_default())
    //     .collect();
    // Ok((rest, parsed_vec))
    // .map(|entries| |(_, text, comment, variation)| ParsedMove::new(text, comment, variation))
    // .context("Parsing all moves")
}

fn variation(
    input: &str,
    // parent: ParsedMove<'a>,
) -> IResult<&str, Vec<ParsedMove<'_>>, ErrorTree<&str>> {
    ws(delimited(char('('), many1(move_entry), char(')')))
        .context("variation")
        .parse(input)
    // {
    //     Ok((rest, parsed)) => Ok((rest, parsed)),
    //     // Err(Err::Error(_)) => Ok((input, vec![])),
    //     Err(e) => Err(e),
    // }

    // Ok((rest, parent))
}

fn move_number_black(input: &str) -> IResult<&str, Option<&str>, ErrorTree<&str>> {
    ws(u8.terminated(tag("...")).recognize().opt()).parse(input)
}

fn move_number_white(input: &str) -> IResult<&str, Option<&str>, ErrorTree<&str>> {
    ws(u8.terminated(tag(".")).recognize().opt()).parse(input)
}

fn move_number(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    alt((
        ws(u8.terminated(tag("..."))).recognize(),
        ws(u8.terminated(tag("."))).recognize(),
    ))
    .parse(input)
    // ws(u8
    //     .terminated(tag("...").or(u8.terminated(tag("."))))
    //     .recognize())
    // .parse(input)
}

pub fn pgn(input: &str) -> IResult<&str, Vec<ParsedMove>, ErrorTree<&str>> {
    (fold_move_entry)
        // .terminated(tag("1-0").or(tag("1/2 - 1/2").or(tag("0-1"))))
        .parse(input)
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    // #[test]
    fn parses_variations() {
        let (_, parsed) = variation("(1... e5 (2.d4) 3.d5)").unwrap();
        dbg!(&parsed);
        assert_eq!(parsed.len(), 3);
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
        //             variations: None
        //         }
        //     ]
        // )
    }

    // #[test]
    fn parses_nested_variations() {
        let (_, parsed) = variation("1... e6 (1... e5 2.d4 (2.Nf3) )").unwrap();

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
    fn it_variation() {
        let (_, parsed) = variation("(1... e5 (1... c5))").unwrap();

        dbg!(&parsed);
        // assert_eq!(
        //     parsed,
        //     vec![
        //         vec![ParsedMove {
        //             r#move: "e5",
        //             variations: vec![],
        //             comment: None,
        //         }],
        //         vec![ParsedMove {
        //             r#move: "c5",
        //             variations: vec![],
        //             comment: None,
        //         }]
        //     ]
        // );
    }

    // #[test]
    fn it_move_numbers() {
        let (_, parsed) = move_number("1.").unwrap();
        assert_eq!(parsed, "1.");
    }
    // #[test]
    fn it_move_numbers_black() {
        let (_, parsed) = move_number("1...").unwrap();
        assert_eq!(parsed, "1...");
    }

    #[test]
    fn it_move_entries() {
        let (_, parsed) = fold_move_entry("1.d4 e5").unwrap();
        assert!(parsed.len() == 2);
        // assert_eq!(
        //     parsed,
        //     vec![ParsedMove {
        //         r#move: "d4",
        //         comment: None,
        //         variations: vec![],
        //     }],
        // );
    }

    // #[test]
    fn it_move_entries_without_move_number() {
        let (_, parsed) = fold_move_entry("d4").unwrap();
        assert!(parsed.len() == 1);
        assert_eq!(
            parsed,
            vec![ParsedMove {
                r#move: "d4",
                comment: None,
                variations: vec![],
            }],
        );
    }
    // #[test]
    fn game_with_nested_comment_and_variations() {
        let (_, parsed) =
            pgn("1.e4 {This is a comment} (1.d4 {This is a comment too} 2.e5 (2... Nf6) )")
                .unwrap();
        dbg!(&parsed);
        assert_eq!(parsed.len(), 1);
        assert_eq!(
            parsed,
            vec![ParsedMove {
                r#move: "e4",
                comment: Some("This is a comment"),
                variations: vec![
                    ParsedMove {
                        r#move: "d4",
                        comment: Some("This is a comment too"),
                        variations: vec![ParsedMove {
                            r#move: "Nf6",
                            comment: None,
                            variations: vec![]
                        }]
                    },
                    ParsedMove {
                        r#move: "e5",
                        comment: None,
                        variations: vec![]
                    }
                ]
            }]
        );
        dbg!(parsed);
    }

    #[test]
    fn game_with_comment_and_variations() {
        let (_, parsed) = pgn("1.e4 {This is a comment}(1... e5 2.d4 )").unwrap();
        dbg!(&parsed);
        assert_eq!(parsed.len(), 1);
        assert_eq!(
            parsed,
            vec![ParsedMove {
                r#move: "e4",
                comment: Some("This is a comment"),
                variations: vec![
                    ParsedMove {
                        r#move: "e5",
                        comment: None,
                        variations: vec![]
                    },
                    ParsedMove {
                        r#move: "d4",
                        comment: None,
                        variations: vec![]
                    }
                ]
            }]
        );
    }

    //#[test]
    #[should_panic]
    fn panics_on_rank_outside_bounds() {
        parse_rank("9").unwrap();
    }

    // #[test]
    fn parses_rank() {
        let (_, move_text) = parse_rank("7").unwrap();
        assert_eq!(move_text, "7");
    }

    //#[test]
    fn parses_file() {
        let (_, move_text) = parse_file("b").unwrap();
        assert_eq!(move_text, "b");
    }

    //#[test]
    #[should_panic]
    fn panics_on_invalid_file() {
        parse_file("y").unwrap();
    }

    // #[test]
    fn parses_move_text() {
        let (_, move_text) = move_text("Nd5").unwrap();
        assert_eq!(move_text, "Nd5",)
    }

    // #[test]
    fn parses_move_text_with_disambiguation() {
        let (rest, move_text) = move_text("Nbd5 Nd2").unwrap();
        assert_eq!(move_text, "Nbd5")
    }

    // #[test]
    fn parses_move_text_with_capture() {
        let (_, move_text) = move_text("Nxd5").unwrap();
        assert_eq!(move_text, "Nxd5",)
    }

    #[test]
    fn parses_capture() {
        let (_, move_text) = parse_capture("xe5").unwrap();
        assert_eq!(move_text, "x")
    }

    //#[test]
    fn parses_move_text_with_disambiguated_capture() {
        let (_, move_text) = move_text("Nexd5").unwrap();
        assert_eq!(move_text, "Nd5");
    }

    //#[test]
    fn parses_disambiguated_capture() {
        let (_, move_text) = parse_disambiguated_capture("ex").unwrap();
        assert_eq!(move_text, "ex")
    }

    //#[test]
    fn parses_comments() {
        let (_, comment) = comment("{This is a comment}").unwrap();
        assert_eq!(comment, "This is a comment");
    }

    // #[test]
    #[should_panic]
    fn should_panic_for_invalid_pgn() {
        let res = pgn("1.z@2").unwrap();
        dbg!(&res);
    }

    // #[test]
    fn parses_first_move() {
        let (_, res) = fold_move_entry("1.e4 e5 2.d4 d5 ").unwrap();
        dbg!(&res);
        assert_eq!(res.len(), 4);
        assert_eq!(
            res[0],
            ParsedMove {
                r#move: "e4",
                comment: None,
                variations: vec![],
            }
        );
        assert_eq!(
            res[1],
            ParsedMove {
                r#move: "e5",
                comment: None,
                variations: vec![],
            }
        );
        assert_eq!(
            res[2],
            ParsedMove {
                r#move: "d4",
                comment: None,
                variations: vec![],
            }
        );
        assert_eq!(
            res[3],
            ParsedMove {
                r#move: "d5",
                comment: None,
                variations: vec![],
            }
        );
    }
}
