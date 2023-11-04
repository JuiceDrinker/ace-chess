use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_while;
use nom::bytes::complete::take_while1;
use nom::character::complete::u8;
use nom::IResult;
use nom::Parser;

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

fn parse_move_white(input: &str) -> IResult<&str, ParsedMove> {
    let (input, _) = u8(input.trim_start())?;
    let (input, _) = tag(".")(input)?;

    let (rest, _) = take_while(|c| c == ' ')(input)?;
    let (rest, r#move) = take_while1(|c: char| c.is_alphanumeric()).parse(rest)?;

    Ok((rest, ParsedMove::new(r#move)))
}

fn parse_move_black(input: &str) -> IResult<&str, ParsedMove> {
    let (rest, r#move) = take_while(|c: char| c.is_alphanumeric()).parse(input.trim_start())?;
    Ok((rest, ParsedMove::new(r#move)))
}

fn parse_pgn(input: &str) -> IResult<&str, ParsedMove> {
    alt((parse_move_white, parse_move_black))(input)
}

fn parse_game(input: &str) -> Vec<ParsedMove> {
    let mut moves = vec![];
    let mut left_to_parse = input;
    while !left_to_parse.is_empty() {
        match parse_pgn(left_to_parse) {
            Ok((rest, parsed_move)) => {
                moves.push(parsed_move);
                left_to_parse = rest;
            }
            Err(_) => {
                break;
            }
        }
    }

    moves
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_first_move() {
        let res = parse_game("1. e4 e5 2. d4 d5");
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
