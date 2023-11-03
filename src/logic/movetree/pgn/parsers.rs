use std::str::FromStr;

use nom::{
    branch::alt,
    bytes::complete::{take_until, take_while_m_n},
    character::complete::{multispace0, u8},
    combinator::{map, opt, recognize},
    error::ParseError,
    sequence::{delimited, tuple},
    IResult, Parser,
};
use nom_supreme::{error::ErrorTree, tag::complete::tag, ParserExt};

use super::Nag;

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

pub fn nag(input: &str) -> IResult<&str, Nag, ErrorTree<&str>> {
    map(
        ws(tag("!?")
            .or(tag("?!"))
            .or(tag("??"))
            .or(tag("?"))
            .or(tag("!!"))
            .or(tag("!"))),
        |nag| Nag::from_str(nag).unwrap(),
    )
    .parse(input)
}
pub fn move_text(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
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

pub fn comment(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    let (rest, comment) =
        ws(delimited(tag("{"), take_until("}"), tag("}"))).parse(input.trim_start())?;
    Ok((rest, comment))
}

fn ws<'a, O, E: ParseError<&'a str>, F: Parser<&'a str, O, E>>(f: F) -> impl Parser<&'a str, O, E> {
    delimited(multispace0, f, multispace0)
}

fn result(input: &str) -> IResult<&str, bool, ErrorTree<&str>> {
    map(tag("1-0").or(tag("0-1").or(tag("1/2-1/2"))), |_| true).parse(input)
}

fn move_number_black(input: &str) -> IResult<&str, Option<&str>, ErrorTree<&str>> {
    ws(u8.terminated(tag("...")).recognize().opt()).parse(input)
}

fn move_number_white(input: &str) -> IResult<&str, Option<&str>, ErrorTree<&str>> {
    ws(u8.terminated(tag(".")).recognize().opt()).parse(input)
}

pub fn move_number(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    alt((
        ws(u8.terminated(tag("..."))).recognize(),
        ws(u8.terminated(tag("."))).recognize(),
    ))
    .parse(input)
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_move_numbers() {
        let (_, parsed) = move_number("1.").unwrap();
        assert_eq!(parsed, "1.");
    }

    #[test]
    fn it_move_numbers_black() {
        let (_, parsed) = move_number("1...").unwrap();
        assert_eq!(parsed, "1...");
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
        let (_, move_text) = move_text("Nd5").unwrap();
        assert_eq!(move_text, "Nd5",)
    }

    #[test]
    fn parses_move_text_with_disambiguation() {
        let (_, move_text) = move_text("Nbd5 Nd2").unwrap();
        assert_eq!(move_text, "Nbd5")
    }

    #[test]
    fn parses_move_text_with_capture() {
        let (_, move_text) = move_text("Nxd5").unwrap();
        assert_eq!(move_text, "Nxd5",)
    }

    #[test]
    fn parses_capture() {
        let (_, move_text) = parse_capture("xe5").unwrap();
        assert_eq!(move_text, "x")
    }

    #[test]
    fn parses_move_text_with_disambiguated_capture() {
        let (_, move_text) = move_text("Nexd5").unwrap();
        assert_eq!(move_text, "Nexd5");
    }

    #[test]
    fn parses_disambiguated_capture() {
        let (_, move_text) = parse_disambiguated_capture("ex").unwrap();
        assert_eq!(move_text, "ex")
    }

    #[test]
    fn parses_nag_interesting() {
        let (_, nag) = nag("!?").unwrap();
        assert_eq!(nag, Nag::Interesting);
    }

    #[test]
    fn parses_nag_dubious() {
        let (_, nag) = nag("?!").unwrap();
        assert_eq!(nag, Nag::Dubious);
    }

    #[test]
    fn parses_nag_poor() {
        let (_, nag) = nag("?").unwrap();
        assert_eq!(nag, Nag::Poor);
    }

    #[test]
    fn parses_nag_blunder() {
        let (_, nag) = nag("??").unwrap();
        assert_eq!(nag, Nag::Blunder);
    }

    #[test]
    fn parses_nag_good() {
        let (_, nag) = nag("!").unwrap();
        assert_eq!(nag, Nag::Good);
    }

    #[test]
    fn parses_nag_excellent() {
        let (_, nag) = nag("!!").unwrap();
        assert_eq!(nag, Nag::Excellent);
    }

    #[test]
    fn parses_comments() {
        let (_, comment) = comment("{This is a comment}").unwrap();
        assert_eq!(comment, "This is a comment");
    }
}
