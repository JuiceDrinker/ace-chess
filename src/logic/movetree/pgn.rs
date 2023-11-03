#[allow(unused)]
use nom::bytes::complete::tag;
use nom::bytes::complete::take_while1;
use nom::character::complete::multispace1;
use nom::character::complete::u8;
use nom::IResult;

fn parse_move_text(input: &str) -> IResult<&str, &str> {
    let (input, move_number) = u8(input)?;
    dbg!(input);
    dbg!(move_number);
    let (input, _) = tag(".")(input)?;

    dbg!(input);
    // take_until1(|c: char| c.is_alphabetic())(input)?;
    let (rest, input) = take_while1(|c| c == ' ')(input)?;
    dbg!(rest);
    // let (rest, whitespace) = multispace1(rest)?;
    let (rest, white_move) = take_while1(|c: char| c.is_alphanumeric())(rest)?;

    let (rest, whitespace) = multispace1(rest)?;
    let (rest, black_move) = take_while1(|c: char| c.is_alphanumeric())(rest)?;

    dbg!(black_move);
    dbg!(white_move);
    Ok((input, rest))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_first_move() {
        let _ = parse_move_text("1. e4 e5 2. d4 d5");
    }

    #[test]
    fn parses_any_move_number() {
        let _ = parse_move_text("2. d4 d5");
    }
}
