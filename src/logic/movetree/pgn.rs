#[allow(unused)]
use nom::bytes::complete::tag;
use nom::bytes::complete::take_while1;
use nom::character::complete::multispace1;
use nom::character::complete::u8;
use nom::IResult;

#[derive(Debug, PartialEq)]
struct ParsedMove<'a> {
    black_move: &'a str,
    white_move: &'a str,
    variations: Option<Box<ParsedMove<'a>>>,
    comment: Option<&'a str>,
}

impl<'a> ParsedMove<'a> {
    fn new(black_move: &'a str, white_move: &'a str) -> ParsedMove<'a> {
        ParsedMove {
            black_move,
            white_move,
            variations: None,
            comment: None,
        }
    }
}

fn parse_move_text(input: &str) -> IResult<&str, ParsedMove> {
    let (input, move_number) = u8(input.trim_start())?;
    let (input, _) = tag(".")(input)?;

    let (rest, input) = take_while1(|c| c == ' ')(input)?;
    let (rest, white_move) = take_while1(|c: char| c.is_alphanumeric())(rest)?;

    let (rest, _) = multispace1(rest)?;
    let (rest, black_move) = take_while1(|c: char| c.is_alphanumeric())(rest)?;

    dbg!((black_move, white_move));
    Ok((rest, ParsedMove::new(black_move, white_move)))
}

fn parse_pgn(input: &str) -> Vec<ParsedMove> {
    let mut moves = vec![];
    let mut left_to_parse = input;
    while !input.is_empty() {
        match parse_move_text(left_to_parse) {
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
        let res = parse_pgn("1. e4 e5 2. d4 d5");
        assert_eq!(res.len(), 2);
        assert_eq!(
            res[0],
            ParsedMove {
                white_move: "e4",
                black_move: "e5",
                comment: None,
                variations: None,
            }
        );
        assert_eq!(
            res[1],
            ParsedMove {
                white_move: "d4",
                black_move: "d5",
                comment: None,
                variations: None,
            }
        );
    }
}
