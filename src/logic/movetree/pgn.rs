#![allow(dead_code)]
use nom::{
    branch::alt,
    bytes::complete::{take_until, take_while_m_n},
    character::complete::{multispace1, u8},
    combinator::{opt, recognize},
    sequence::{delimited, tuple, Tuple},
    IResult, Parser,
};
use nom_supreme::{error::ErrorTree, tag::complete::tag, ParserExt};

#[derive(Debug, PartialEq)]
pub struct ParsedMove<'a> {
    r#move: &'a str,
    variations: Option<Vec<ParsedMove<'a>>>,
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
            variations,
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

fn parse_basic_pawn_move(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    recognize(tuple((parse_file, parse_rank)))
        .context("basic pawn move")
        .parse(input)
}

fn parse_pawn_capture(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    recognize(tuple((parse_file, tag("x"), parse_rank)))
        .context("pawn capture")
        .parse(input)
}

fn parse_pawn_promotion(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    recognize(tuple((
        parse_file,
        opt(tag("x")),
        parse_rank,
        tag("="),
        parse_promotion_piece,
    )))
    .parse(input)
}

fn parse_pawn_move(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    alt((
        parse_basic_pawn_move,
        parse_pawn_capture,
        parse_pawn_capture,
    ))
    .parse(input)
}

fn parse_move_text(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    alt((parse_pawn_move, parse_piece_move, tag("0-0"), tag("0-0-0"))).parse(input)
}

fn parse_piece_move(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    // Nd5, Nbd5, Nbxd5, Nxd5
    alt((parse_basic, parse_disambugated_move, parse_full_capture)).parse(input)
}

fn parse_move_number_black(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    recognize(tuple((u8, tag("... "))))
        .context("Parsing blacks move number")
        .parse(input)
}

fn parse_move_number_white(input: &str) -> IResult<&str, (u8, &str), ErrorTree<&str>> {
    (u8, tag(".")).parse(input)
}

fn parse_move_white(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    let (rest, _) = parse_move_number_white(input.trim_start())?;
    let (rest, r#move) = parse_move_text.parse(rest)?;

    Ok((rest, r#move))
}

fn parse_comments(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    let (rest, comment) =
        delimited(tag("{"), take_until("}"), tag("}")).parse(input.trim_start())?;
    Ok((rest, comment))
}
fn parse_move_black(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    let (rest, _) = opt(alt((multispace1, parse_move_number_black)))
        .context("Discard move number for black")
        .parse(input)?;
    let (rest, r#move) = parse_move_text.parse(rest)?;
    Ok((rest, r#move))
}

fn parse_move(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    alt((parse_move_black, parse_move_white))
        .context("parsing move")
        .parse(input)
}

fn parse_variations(input: &str) -> IResult<&str, Vec<ParsedMove>, ErrorTree<&str>> {
    let (_, moves) = delimited(tag("("), take_until(")"), tag(")"))(input)?;

    dbg!(moves);
    let mut moves_left_to_parse = moves;
    let mut variations = vec![];

    while !moves_left_to_parse.is_empty() {
        let (rest, parsed_move) = parse_move_with_comment(moves_left_to_parse)?;
        variations.push(parsed_move);
        moves_left_to_parse = rest;
    }
    Ok((moves_left_to_parse, variations))
}

fn parse_move_with_comment(input: &str) -> IResult<&str, ParsedMove, ErrorTree<&str>> {
    let (rest, parsed_move) = parse_move(input)?;
    let (rest, _) = opt(multispace1).parse(rest)?;
    let (rest, comment) = opt(parse_comments).parse(rest)?;
    let (rest, variations) = opt(parse_variations).parse(rest)?;

    Ok((rest, ParsedMove::new(parsed_move, comment, variations)))
}

pub fn parse_pgn(input: &str) -> IResult<&str, Vec<ParsedMove>, ErrorTree<&str>> {
    let mut moves = vec![];
    let mut left_to_parse = input;
    while !left_to_parse.is_empty() {
        let (rest, parsed_move) = parse_move_with_comment(left_to_parse)?;
        moves.push(parsed_move);
        left_to_parse = rest;
    }

    Ok((left_to_parse, moves))
}

#[cfg(test)]
mod test {
    use super::*;

    // #[test]
    fn parses_variations() {
        let (_, parsed) = parse_variations("(1... e5 2.d4 )").unwrap();
        assert_eq!(
            parsed,
            vec![
                ParsedMove {
                    r#move: "e5",
                    comment: None,
                    variations: None
                },
                ParsedMove {
                    r#move: "d4",
                    comment: None,
                    variations: None
                }
            ]
        )
    }

    #[test]
    fn parses_nested_variations() {
        let (_, parsed) = parse_variations("(1... e5 2.d4 (2. Nf3) )").unwrap();
        assert_eq!(
            parsed,
            vec![
                ParsedMove {
                    r#move: "e5",
                    comment: None,
                    variations: None
                },
                ParsedMove {
                    r#move: "d4",
                    comment: None,
                    variations: None
                }
            ]
        )
    }

    // #[test]
    fn game_with_nested_comment_and_variations() {
        let (_, parsed) =
            parse_pgn("1.e4 {This is a comment}(1... e5 {This is a comment too} 2.d4 (2. Nf3) )")
                .unwrap();
        // assert_eq!(parsed.len(), 1);
        // assert_eq!(
        //     parsed,
        //     vec![ParsedMove {
        //         r#move: "e4",
        //         comment: Some("This is a comment"),
        //         variations: Some(vec![
        //             ParsedMove {
        //                 r#move: "e5",
        //                 comment: None,
        //                 variations: None
        //             },
        //             ParsedMove {
        //                 r#move: "d4",
        //                 comment: None,
        //                 variations: None
        //             }
        //         ])
        //     }]
        // );
        dbg!(parsed);
    }
    #[test]
    fn game_with_comment_and_variations() {
        let (_, parsed) = parse_pgn("1.e4 {This is a comment}(1... e5 2.d4 )").unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(
            parsed,
            vec![ParsedMove {
                r#move: "e4",
                comment: Some("This is a comment"),
                variations: Some(vec![
                    ParsedMove {
                        r#move: "e5",
                        comment: None,
                        variations: None
                    },
                    ParsedMove {
                        r#move: "d4",
                        comment: None,
                        variations: None
                    }
                ])
            }]
        );
    }

    #[test]
    fn move_without_comment() {
        let (_, parsed) = parse_move_with_comment("1.e4 ").unwrap();
        assert_eq!(
            parsed,
            ParsedMove {
                r#move: "e4",
                comment: None,
                variations: None,
            }
        );
    }

    #[test]
    #[should_panic]
    fn panics_on_rank_outside_bounds() {
        parse_rank("9").unwrap();
    }

    #[test]
    fn parses_rank() {
        let (_, move_text) = parse_rank("7").unwrap();
        assert_eq!(move_text, "7");
    }

    #[test]
    fn parses_file() {
        let (_, move_text) = parse_file("b").unwrap();
        assert_eq!(move_text, "b");
    }

    #[test]
    #[should_panic]
    fn panics_on_invalid_file() {
        parse_file("y").unwrap();
    }

    #[test]
    fn parses_move_text() {
        let (_, move_text) = parse_move_text("Nd5").unwrap();
        assert_eq!(move_text, "Nd5")
    }

    #[test]
    fn parses_move_text_with_disambiguation() {
        let (_, move_text) = parse_move_text("Nbd5").unwrap();
        assert_eq!(move_text, "Nbd5")
    }

    #[test]
    fn parses_move_text_with_capture() {
        let (_, move_text) = parse_move_text("Nxd5").unwrap();
        assert_eq!(move_text, "Nxd5")
    }

    #[test]
    fn parses_capture() {
        let (_, move_text) = parse_capture("xe5").unwrap();
        assert_eq!(move_text, "x")
    }

    #[test]
    fn parses_move_text_with_disambiguated_capture() {
        let (_, move_text) = parse_move_text("Nexd5").unwrap();
        assert_eq!(move_text, "Nexd5")
    }

    #[test]
    fn parses_disambiguated_capture() {
        let (_, move_text) = parse_disambiguated_capture("ex").unwrap();
        assert_eq!(move_text, "ex")
    }

    #[test]
    fn parses_comments() {
        let (_, comment) = parse_comments("{This is a comment}").unwrap();
        assert_eq!(comment, "This is a comment");
    }

    #[test]
    fn parses_move_number() {
        let res = parse_move_number_white("1.").unwrap();
        assert_eq!(res, ("", (1, ".")));
    }

    #[test]
    fn parses_any_move_number() {
        let res = parse_move_number_white("8.").unwrap();
        assert_eq!(res, ("", (8, ".")));
    }

    #[test]
    fn parses_whites_move() {
        let res = parse_move_white("1.e4").unwrap();
        assert_eq!(res, ("", "e4"));
    }

    #[test]
    fn parses_black_move() {
        let res = parse_move_black(" e5").unwrap();
        assert_eq!(res, ("", "e5"));
    }

    #[test]
    fn parses_blacks_move_with_move_number() {
        let res = parse_move_black("2... e5").unwrap();
        assert_eq!(res, ("", "e5"));
    }

    #[test]
    #[should_panic]
    fn should_panic_for_invalid_pgn() {
        parse_pgn("1 .e4").unwrap();
    }

    #[test]
    fn parses_first_move() {
        let (_, res) = parse_pgn("1.e4 e5 2.d4 d5").unwrap();
        assert_eq!(res.len(), 4);
        assert_eq!(
            res[0],
            ParsedMove {
                r#move: "e4",
                comment: None,
                variations: None,
            }
        );
        assert_eq!(
            res[1],
            ParsedMove {
                r#move: "e5",
                comment: None,
                variations: None,
            }
        );
        assert_eq!(
            res[2],
            ParsedMove {
                r#move: "d4",
                comment: None,
                variations: None,
            }
        );
        assert_eq!(
            res[3],
            ParsedMove {
                r#move: "d5",
                comment: None,
                variations: None,
            }
        );
    }
}
