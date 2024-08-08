use crate::common::piece::Piece;

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    let mut chars = input.char_indices().peekable();

    while let Some((idx, char)) = chars.next() {
        let token = match char {
            ' ' | '\n' | '\t' => continue,
            '/' => Token::Slash,
            '*' => Token::Star,
            '.' => Token::Dot,
            '(' => Token::StartVariation,
            ')' => Token::EndVariation,
            '{' => {
                let comment: String = chars
                    .by_ref()
                    .take_while(|(_, char)| *char != '}')
                    .map(|(_, char)| char)
                    .collect();
                Token::Comment(comment)
            }
            '0'..='9' => Token::Number(char.to_digit(10).unwrap()),
            'a'..='h' => Token::File(char),
            '=' => Token::Equals,
            '#' => Token::Checkmate,
            '+' => Token::Check,
            'x' => Token::Captures,
            'N' => Token::Piece(Piece::Knight),
            'B' => Token::Piece(Piece::Bishop),
            'K' => Token::Piece(Piece::King),
            'Q' => Token::Piece(Piece::Queen),
            'R' => Token::Piece(Piece::Rook),
            '-' => Token::Hyphen,
            '?' => {
                if chars.next_if_eq(&(idx + 1, '?')).is_some() {
                    Token::Nag(Nag::Blunder)
                } else if chars.next_if_eq(&(idx + 1, '!')).is_some() {
                    Token::Nag(Nag::Dubious)
                } else {
                    Token::Nag(Nag::Poor)
                }
            }
            '!' => {
                if chars.next_if_eq(&(idx + 1, '!')).is_some() {
                    Token::Nag(Nag::Excellent)
                } else if chars.next_if_eq(&(idx + 1, '?')).is_some() {
                    Token::Nag(Nag::Interesting)
                } else {
                    Token::Nag(Nag::Good)
                }
            }
            _ => Token::Invalid,
        };
        tokens.push(token);
    }
    tokens
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Token {
    Star,
    Slash,
    Number(u32),
    File(char),
    Piece(Piece),
    Comment(String),
    Nag(Nag),
    Hyphen,
    Equals,
    Dot,
    Captures,
    Check,
    Checkmate,
    StartVariation,
    EndVariation,
    Invalid,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Nag {
    Good,
    Excellent,
    Interesting,
    Blunder,
    Poor,
    Dubious,
}
