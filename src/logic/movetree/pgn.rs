#![allow(dead_code)]
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{digit1, multispace1, u8},
    combinator::{map_res, opt, recognize, verify},
    sequence::{delimited, tuple, Tuple},
    IResult, Parser,
};

#[derive(Debug, PartialEq)]
struct ParsedMove<'a> {
    r#move: &'a str,
    variations: Option<Box<ParsedMove<'a>>>,
    comment: Option<&'a str>,
}

impl<'a> ParsedMove<'a> {
    fn new(r#move: &'a str) -> ParsedMove<'a> {
        ParsedMove {
            r#move,
            variations: None,
            comment: None,
        }
    }
}

fn parse_rank(input: &str) -> IResult<&str, &str> {
    // dbg!(input);
    take_while1(|c: char| matches!(c, '1'..='8')).parse(input)
    // dbg!();
    // let (a, b) = map_res(digit1, |s: &str| s.parse::<u8>())(input)?;

    // Ok((a, a))
    // Ok()
}

fn parse_file(input: &str) -> IResult<&str, &str> {
    dbg!(input);
    take_while1(|c: char| matches!(c, 'a'..='h')).parse(input)
}
fn parse_piece(input: &str) -> IResult<&str, &str> {
    dbg!(input);
    take_while1(|c: char| matches!(c, 'N' | 'B' | 'R' | 'Q' | 'K')).parse(input)
}

// fn parse_disambiguated_capture(input: &str) -> IResult<&str, &str> {
//     recognize(tuple((parse_file, tag("x")))).parse(input)
// }
fn parse_capture(input: &str) -> IResult<&str, &str> {
    dbg!(input);
    tag("x").parse(input)
    // alt((tag("x"), parse_disambiguated_capture)).parse(input)
}

fn parse_move_text(input: &str) -> IResult<&str, &str> {
    recognize(tuple((
        parse_piece,
        opt(parse_file),
        opt(parse_capture),
        parse_file,
        parse_rank,
    )))
    .parse(input)
}
fn parse_move_number_black(input: &str) -> IResult<&str, &str> {
    let (rest, (move_number, tag)) = (u8, tag("... ")).parse(input)?;
    Ok((rest, tag))
}
fn parse_move_number_white(input: &str) -> IResult<&str, (u8, &str)> {
    (u8, tag(".")).parse(input)
}

fn parse_move_white(input: &str) -> IResult<&str, ParsedMove> {
    let (rest, _) = parse_move_number_white(input.trim_start())?;
    let (rest, r#move) = take_while1(|c: char| c.is_alphanumeric()).parse(rest)?;

    Ok((rest, ParsedMove::new(r#move)))
}

fn parse_comments(input: &str) -> IResult<&str, &str> {
    delimited(tag("{"), take_until("}"), tag("}")).parse(input)
}
fn parse_move_black(input: &str) -> IResult<&str, ParsedMove> {
    let (rest, _) = alt((multispace1, parse_move_number_black)).parse(input)?;
    let (rest, r#move) = take_while1(|c: char| c.is_alphanumeric()).parse(rest)?;
    Ok((rest, ParsedMove::new(r#move)))
}

fn parse_pgn(input: &str) -> IResult<&str, ParsedMove> {
    alt((parse_move_white, parse_move_black))(input)
}

fn parse_game(input: &str) -> Result<Vec<ParsedMove>, nom::Err<nom::error::Error<&str>>> {
    let mut moves = vec![];
    let mut left_to_parse = input;
    while !left_to_parse.is_empty() {
        let (rest, parsed_move) = parse_pgn(left_to_parse)?;
        moves.push(parsed_move);
        left_to_parse = rest;
    }

    Ok(moves)
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_move_text() {
        let (_, move_text) = parse_move_text("Nd5").unwrap();
        assert_eq!(move_text, "Nd5")
    }

    // #[test]
    fn parses_move_text_with_disambiguation() {
        let (_, move_text) = parse_move_text("Nbd5").unwrap();
        assert_eq!(move_text, "Nbd5")
    }

    // #[test]
    fn parses_move_text_with_capture() {
        let (_, move_text) = parse_move_text("Nxd5").unwrap();
        assert_eq!(move_text, "Nxd5")
    }

    // #[test]
    fn parses_capture() {
        let (_, move_text) = parse_capture("xe5").unwrap();
        assert_eq!(move_text, "x")
    }

    // #[test]
    fn parses_move_text_with_disambiguated_capture() {
        let (_, move_text) = parse_move_text("Nexd5").unwrap();
        assert_eq!(move_text, "Nexd5")
    }

    // #[test]
    // fn parses_disambiguated_capture() {
    //     let (_, move_text) = parse_disambiguated_capture("ex").unwrap();
    //     assert_eq!(move_text, "ex")
    // }
    // #[test]
    fn parses_comments() {
        let (_, comment) = parse_comments("{This is a comment}").unwrap();
        assert_eq!(comment, "This is a comment");
    }
    // #[test]
    fn parses_move_number() {
        let res = parse_move_number_white("1.").unwrap();
        assert_eq!(res, ("", (1, ".")));
    }

    // #[test]
    fn parses_any_move_number() {
        let res = parse_move_number_white("8.").unwrap();
        assert_eq!(res, ("", (8, ".")));
    }

    // #[test]
    fn parses_whites_move() {
        let res = parse_move_white("1.e4").unwrap();
        assert_eq!(
            res,
            (
                "",
                ParsedMove {
                    r#move: "e4",
                    variations: None,
                    comment: None,
                }
            )
        );
    }

    // #[test]
    fn parses_black_move() {
        let res = parse_move_black(" e5").unwrap();
        assert_eq!(
            res,
            (
                "",
                ParsedMove {
                    r#move: "e5",
                    variations: None,
                    comment: None,
                }
            )
        );
    }

    // #[test]
    fn parses_blacks_move_with_move_number() {
        let res = parse_move_black("2... e5").unwrap();
        assert_eq!(
            res,
            (
                "",
                ParsedMove {
                    r#move: "e5",
                    variations: None,
                    comment: None,
                }
            )
        );
    }

    // #[test]
    #[should_panic]
    fn should_panic_for_invalid_pgn() {
        parse_game("1 .e4").unwrap();
    }

    // #[test]
    fn parses_first_move() {
        let res = parse_game("1.e4 e5 2.d4 d5").unwrap();
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
