#![allow(dead_code)]
use std::{
    fmt::{Debug, Display},
    iter::Peekable,
    slice::Iter,
    str::FromStr,
};

use crate::{
    common::{color::Color, file::File, piece::Piece, rank::Rank},
    logic::movetree::{
        treenode::{CMove, CMoveKind, CResult, CastleSide, MoveDetails, TreeNode},
        MoveTree,
    },
};

const STARTING_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq - 0 1";
// Grammar
// R: 1 … 8               # Rank
// F: a … h               # File
// P: N, B, R, Q, K       # Piece
// PM: PFFR | PRFR | PFR | PC # Piece Move
// PC: P'x'FR | PFxFR | PRxFR # Piece Move
// PM1: FR | FxFR | FR=P | FxFR=P  # Pawn Move
// C: { string }          # Comment
// C1: '0-0' | '0-0-0'    # Castling
// MN: [0-9]+             # Move Number
// D: .                   # Dot
// CH: + | #              # Check/Checkmate
// M: (PM | PM1 | C1) CH?  # Move (with optional check/checkmate)
// MT: M | MN D M | MN DDD M  # Move Text
// V: ( E )               # Variation
// E: MT C | V | E E    # Element (allows for comments and variations between moves)
// R: '1-0' | '0-1' | '1/2-1/2' | '*'  # Result
// TS: '[' string string ']'  # Tag Section
// G: TS* E* R           # Game (with optional tags, multiple elements, and result)

#[derive(Debug)]
pub struct PgnParser<'a> {
    tokens: Peekable<Iter<'a, Token>>,
    cursor: usize,
}

pub enum Expression {
    Move(CMove),
    Variation(Vec<Expression>),
    Sequence(Box<Expression>, Box<Expression>),
}

#[derive(Debug, PartialEq, Eq)]
enum MoveNumber {
    WhiteMoveNumber(String),
    BlackMoveNumber(String),
}

impl<'a> PgnParser<'a> {
    fn new(tokens: Iter<'a, Token>) -> Self {
        Self {
            tokens: tokens.peekable(),
            cursor: 0,
        }
    }

    fn parse(&mut self) -> Result<MoveTree, PgnParseError> {
        let mut move_tree = MoveTree::new();

        let root = move_tree.0.new_node(TreeNode::GameStart);
        let mut current_fen = String::from(STARTING_POSITION_FEN);
        let mut current = root;

        while let Ok(expression) = self.expression() {
            let (node, fen) =
                move_tree.add_expression_to_tree(expression, &current, current_fen.clone())?;
            if !matches!(move_tree.0[node].get(), TreeNode::EndVariation) {
                current = node;
                current_fen = fen;
            }
        }

        let result = self.result()?;
        let new_node = move_tree.0.new_node(TreeNode::Result(result));
        current.append(new_node, &mut move_tree.0);

        Ok(move_tree)
    }

    // E: MT C? | V | E E    # Element (allows for comments and variations between moves)
    fn expression(&mut self) -> Result<Expression, PgnParseError> {
        let first_expression = if let Ok(mut move_text) = self.move_text() {
            if let Ok(comment) = self.comment() {
                move_text.comment = Some(comment);
                Expression::Move(move_text)
            } else {
                Expression::Move(move_text)
            }
        } else if let Ok(variation) = self.variation() {
            Expression::Variation(variation)
        } else {
            return Err(PgnParseError::syntax(self.cursor));
        };

        if let Ok(second_expression) = self.expression() {
            Ok(Expression::Sequence(
                Box::new(first_expression),
                Box::new(second_expression),
            ))
        } else {
            Ok(first_expression)
        }
    }

    fn variation(&mut self) -> Result<Vec<Expression>, PgnParseError> {
        if self.tokens.peek().is_none() {
            return Err(PgnParseError::unexpected_eof(self.cursor));
        };

        let mut vec = vec![];
        if let Some(Token::StartVariation) = self.tokens.peek() {
            self.consume();
            loop {
                if let Some(Token::EndVariation) = self.tokens.peek() {
                    self.consume();
                    return Ok(vec);
                } else if let Ok(expression) = self.expression() {
                    vec.push(expression);
                }
            }
        }

        Err(PgnParseError::syntax(self.cursor))
    }
    fn consume(&mut self) {
        self.cursor += 1;
        self.tokens.next();
    }

    fn result(&mut self) -> Result<CResult, PgnParseError> {
        let iter_save = self.tokens.clone();
        let cursor_save = self.cursor;

        match self.tokens.peek() {
            Some(Token::Number(1)) => {
                self.consume();
                match self.tokens.peek() {
                    Some(Token::Hyphen) => {
                        self.consume();
                        if let Some(Token::Number(0)) = self.tokens.peek() {
                            self.consume();
                            Ok(CResult::WhiteWins)
                        } else {
                            self.tokens = iter_save;
                            self.cursor = cursor_save;
                            Err(PgnParseError::syntax(self.cursor))
                        }
                    }
                    Some(Token::Slash) => {
                        self.consume();
                        // This is a bit ugly... also white space??
                        if let Some(Token::Number(2)) = self.tokens.peek() {
                            self.consume();
                            if let Some(Token::Hyphen) = self.tokens.peek() {
                                self.consume();
                                if let Some(Token::Number(1)) = self.tokens.peek() {
                                    self.consume();
                                    if let Some(Token::Slash) = self.tokens.peek() {
                                        self.consume();
                                        if let Some(Token::Number(2)) = self.tokens.peek() {
                                            self.consume();
                                            return Ok(CResult::Draw);
                                        }
                                    }
                                }
                            }
                        }

                        self.tokens = iter_save;
                        self.cursor = cursor_save;
                        Err(PgnParseError::syntax(self.cursor))
                    }
                    _ => {
                        self.tokens = iter_save;
                        self.cursor = cursor_save;
                        Err(PgnParseError::syntax(self.cursor))
                    }
                }
            }
            Some(Token::Number(0)) => {
                self.consume();
                if let Some(Token::Hyphen) = self.tokens.peek() {
                    self.consume();
                    if let Some(Token::Number(1)) = self.tokens.peek() {
                        self.consume();
                        Ok(CResult::BlackWins)
                    } else {
                        self.tokens = iter_save;
                        self.cursor = cursor_save;
                        Err(PgnParseError::syntax(self.cursor))
                    }
                } else {
                    self.tokens = iter_save;
                    self.cursor = cursor_save;
                    Err(PgnParseError::syntax(self.cursor))
                }
            }
            Some(Token::Star) => {
                self.consume();
                Ok(CResult::NoResult)
            }
            Some(_) => Err(PgnParseError::syntax(self.cursor)),
            None => Err(PgnParseError::unexpected_eof(self.cursor)),
        }
    }
    fn comment(&mut self) -> Result<String, PgnParseError> {
        match self.tokens.peek() {
            Some(Token::Comment(comment)) => {
                self.consume();
                Ok(comment.to_string())
            }
            Some(_) => Err(PgnParseError::syntax(self.cursor)),
            None => Err(PgnParseError::unexpected_eof(self.cursor)),
        }
    }

    fn checkmate(&mut self) -> Result<String, PgnParseError> {
        match self.tokens.peek() {
            Some(Token::Checkmate) => {
                self.consume();
                Ok("#".to_string())
            }
            Some(_) => Err(PgnParseError::syntax(self.cursor)),
            None => Err(PgnParseError::unexpected_eof(self.cursor)),
        }
    }

    fn check(&mut self) -> Result<String, PgnParseError> {
        match self.tokens.peek() {
            Some(Token::Check) => {
                self.consume();
                Ok("+".to_string())
            }
            Some(_) => Err(PgnParseError::syntax(self.cursor)),
            None => Err(PgnParseError::unexpected_eof(self.cursor)),
        }
    }

    fn dot(&mut self) -> Result<String, PgnParseError> {
        match self.tokens.peek() {
            Some(Token::Dot) => {
                self.consume();
                Ok(".".to_string())
            }
            Some(_) => Err(PgnParseError::syntax(self.cursor)),
            None => Err(PgnParseError::unexpected_eof(self.cursor)),
        }
    }

    // MT: M | MN D M | MN DDD M  # Move Text
    fn move_text(&mut self) -> Result<CMove, PgnParseError> {
        if let Ok(mut move_kind) = self.r#move() {
            move_kind.color = Color::Black;
            return Ok(move_kind);
        } else if let Ok(move_number) = self.move_number() {
            if let Ok(mut move_kind) = self.r#move() {
                return match move_number {
                    MoveNumber::WhiteMoveNumber(_) => Ok(move_kind),
                    MoveNumber::BlackMoveNumber(_) => {
                        move_kind.color = Color::Black;
                        Ok(move_kind)
                    }
                };
            }
        }
        Err(PgnParseError::syntax(self.cursor))
    }

    fn move_number(&mut self) -> Result<MoveNumber, PgnParseError> {
        let iter_save = self.tokens.clone();
        let cursor_save = self.cursor;
        match self.tokens.peek() {
            Some(Token::Number(number)) => {
                let mut number = number.to_string();
                self.consume();
                loop {
                    if self.dot().is_ok() {
                        if self.dot().is_ok() {
                            if self.dot().is_ok() {
                                return Ok(MoveNumber::BlackMoveNumber(number));
                            } else {
                                self.tokens = iter_save;
                                self.cursor = cursor_save;
                                return Err(PgnParseError::syntax(self.cursor));
                            }
                        } else {
                            return Ok(MoveNumber::WhiteMoveNumber(number));
                        }
                    } else if let Some(Token::Number(n)) = self.tokens.peek() {
                        number.push_str(&n.to_string());
                        self.consume();
                    } else {
                        self.consume();
                        return Ok(MoveNumber::BlackMoveNumber(number));
                    }
                }
            }
            Some(_) => Err(PgnParseError::syntax(self.cursor)),
            None => Err(PgnParseError::unexpected_eof(self.cursor)),
        }
    }

    // M: (PM | PM1 | C1) CH?  # Move (with optional check/checkmate)
    fn r#move(&mut self) -> Result<CMove, PgnParseError> {
        let move_kind = if let Ok(piece_move) = self.piece_move() {
            piece_move
        } else if let Ok(pawn_move) = self.pawn_move() {
            pawn_move
        } else if let Ok(castle) = self.castle() {
            castle
        } else {
            return Err(PgnParseError::syntax(self.cursor));
        };

        if self.check().is_ok() {
            return Ok(CMove {
                kind: move_kind,
                check: true,
                checkmate: false,
                color: Color::White,
                comment: None,
            });
        } else if self.checkmate().is_ok() {
            return Ok(CMove {
                kind: move_kind,
                check: false,
                checkmate: true,
                color: Color::White,
                comment: None,
            });
        }

        Ok(CMove {
            kind: move_kind,
            check: false,
            checkmate: false,
            color: Color::White,
            comment: None,
        })
    }

    // // PM1: FR | FR=P | FxFR | FxFR=P  # Pawn Move
    fn pawn_move(&mut self) -> Result<CMoveKind, PgnParseError> {
        let iter_save = self.tokens.clone();
        let cursor_save = self.cursor;
        let file = dbg!(self.file())?;

        if let Ok(rank) = self.rank() {
            if self.equals().is_ok() {
                if let Ok(piece) = self.piece() {
                    let cmove = MoveDetails {
                        piece: Piece::Pawn,
                        dst_rank: rank,
                        dst_file: file,
                        captures: false,
                        disam_rank: None,
                        disam_file: None,
                        promotion: Some(piece),
                    };
                    return Ok(CMoveKind::Regular(cmove));
                }
            } else {
                let cmove = MoveDetails {
                    piece: Piece::Pawn,
                    dst_rank: rank,
                    dst_file: file,
                    captures: false,
                    disam_rank: None,
                    disam_file: None,
                    promotion: None,
                };
                return Ok(CMoveKind::Regular(cmove));
            }
        } else if self.captures().is_ok() {
            if let Ok(dst_file) = self.file() {
                if let Ok(rank) = self.rank() {
                    if self.equals().is_ok() {
                        if let Ok(piece) = self.piece() {
                            let cmove = MoveDetails {
                                piece: Piece::Pawn,
                                dst_rank: rank,
                                dst_file,
                                captures: true,
                                disam_rank: None,
                                disam_file: Some(file),
                                promotion: Some(piece),
                            };
                            return Ok(CMoveKind::Regular(cmove));
                        }
                    } else {
                        let cmove = MoveDetails {
                            piece: Piece::Pawn,
                            dst_rank: rank,
                            dst_file,
                            captures: true,
                            disam_rank: None,
                            disam_file: Some(file),
                            promotion: None,
                        };
                        return Ok(CMoveKind::Regular(cmove));
                    }
                }
            }
        }

        self.tokens = iter_save;
        self.cursor = cursor_save;

        Err(PgnParseError::syntax(self.cursor))
    }

    fn equals(&mut self) -> Result<(), PgnParseError> {
        match self.tokens.peek() {
            Some(Token::Equals) => {
                self.consume();
                Ok(())
            }
            Some(_) => Err(PgnParseError::syntax(self.cursor)),
            None => Err(PgnParseError::unexpected_eof(self.cursor)),
        }
    }

    fn castle(&mut self) -> Result<CMoveKind, PgnParseError> {
        match self.tokens.peek() {
            Some(Token::CastleLong) => {
                self.consume();
                Ok(CMoveKind::Castles(CastleSide::Long))
            }
            Some(Token::CastleShort) => {
                self.consume();
                Ok(CMoveKind::Castles(CastleSide::Long))
            }
            Some(_) => Err(PgnParseError::syntax(self.cursor)),
            None => Err(PgnParseError::unexpected_eof(self.cursor)),
        }
    }

    // PM: PFFR | PRFR | PFR | PC # Piece Move
    fn piece_move(&mut self) -> Result<CMoveKind, PgnParseError> {
        let iter_save = self.tokens.clone();
        let cursor_save = self.cursor;

        let piece = self.piece()?;

        if self.tokens.peek().is_none() {
            self.tokens = iter_save;
            self.cursor = cursor_save;
            return Err(PgnParseError::unexpected_eof(self.cursor));
        }

        if let Ok(file) = self.file() {
            // PFFR
            if let Ok(dst_file) = self.file() {
                if let Ok(rank) = self.rank() {
                    let cmove = MoveDetails {
                        piece,
                        dst_rank: rank,
                        dst_file,
                        captures: false,
                        disam_rank: None,
                        disam_file: Some(file),
                        promotion: None,
                    };
                    return Ok(CMoveKind::Regular(cmove));
                }
                // PFR
            } else if let Ok(rank) = self.rank() {
                let cmove = MoveDetails {
                    piece,
                    dst_rank: rank,
                    dst_file: file,
                    captures: false,
                    disam_rank: None,
                    disam_file: None,
                    promotion: None,
                };
                return Ok(CMoveKind::Regular(cmove));
            }

            self.tokens = iter_save;
            self.cursor = cursor_save;
            return Err(PgnParseError::syntax(self.cursor));
        }

        // PRFR
        if let Ok(source_rank) = self.rank() {
            if let Ok(file) = self.file() {
                if let Ok(rank) = self.rank() {
                    let cmove = MoveDetails {
                        piece,
                        dst_rank: rank,
                        dst_file: file,
                        captures: false,
                        disam_rank: Some(source_rank),
                        disam_file: None,
                        promotion: None,
                    };
                    return Ok(CMoveKind::Regular(cmove));
                }
            }
            self.tokens = iter_save;
            self.cursor = cursor_save;
            return Err(PgnParseError::syntax(self.cursor));
        }

        self.tokens = iter_save;
        self.cursor = cursor_save;
        self.piece_capture()
    }

    // PC: P'x'FR | PFxFR | PRxFR # Piece Capture
    fn piece_capture(&mut self) -> Result<CMoveKind, PgnParseError> {
        let iter_save = self.tokens.clone();
        let cursor_save = self.cursor;
        let piece = self.piece()?;

        // Try PxFR
        if self.captures().is_ok() {
            if let Ok(file) = self.file() {
                if let Ok(rank) = self.rank() {
                    let cmove = MoveDetails {
                        piece,
                        dst_rank: rank,
                        dst_file: file,
                        captures: true,
                        disam_rank: None,
                        disam_file: None,
                        promotion: None,
                    };
                    return Ok(CMoveKind::Regular(cmove));
                }
            }
            self.tokens = iter_save;
            self.cursor = cursor_save;
            return Err(PgnParseError::syntax(self.cursor));
        }

        // Try PFxFR
        if let Ok(src_file) = self.file() {
            if self.captures().is_ok() {
                if let Ok(dst_file) = self.file() {
                    if let Ok(rank) = self.rank() {
                        let cmove = MoveDetails {
                            piece,
                            dst_rank: rank,
                            dst_file,
                            captures: true,
                            disam_rank: None,
                            disam_file: Some(src_file),
                            promotion: None,
                        };
                        return Ok(CMoveKind::Regular(cmove));
                    }
                }
            }
            self.tokens = iter_save;
            self.cursor = cursor_save;
            return Err(PgnParseError::syntax(self.cursor));
        }

        // Try PRxFR
        if let Ok(from_rank) = self.rank() {
            if self.captures().is_ok() {
                if let Ok(file) = self.file() {
                    if let Ok(to_rank) = self.rank() {
                        let cmove = MoveDetails {
                            piece,
                            dst_rank: to_rank,
                            dst_file: file,
                            captures: true,
                            disam_rank: Some(from_rank),
                            disam_file: None,
                            promotion: None,
                        };
                        return Ok(CMoveKind::Regular(cmove));
                    }
                }
            }
            self.tokens = iter_save;
            self.cursor = cursor_save;
            return Err(PgnParseError::syntax(self.cursor));
        }

        Err(PgnParseError::syntax(self.cursor))
    }

    // P: N, B, R, Q, K       # Piece
    fn piece(&mut self) -> Result<Piece, PgnParseError> {
        let piece = match self.tokens.peek() {
            Some(Token::Piece(piece)) => {
                self.consume();
                piece
            }
            None => return Err(PgnParseError::unexpected_eof(self.cursor)),
            _ => return Err(PgnParseError::syntax(self.cursor)),
        };

        Ok(piece.clone())
    }

    fn captures(&mut self) -> Result<(), PgnParseError> {
        match self.tokens.peek() {
            Some(Token::Captures) => {
                self.consume();
                Ok(())
            }
            Some(_) => Err(PgnParseError::syntax(self.cursor)),
            None => Err(PgnParseError::unexpected_eof(self.cursor)),
        }
    }

    // 'a' - 'h'
    fn file(&mut self) -> Result<File, PgnParseError> {
        match self.tokens.peek() {
            Some(Token::File(file)) => {
                let file = File::from_str(&file.to_string())
                    .map_err(|_| PgnParseError::syntax(self.cursor))?;
                self.consume();
                Ok(file)
            }

            Some(_) => Err(PgnParseError::syntax(self.cursor)),
            None => Err(PgnParseError::unexpected_eof(self.cursor)),
        }
    }

    // R: 1 … 8               # Rank
    fn rank(&mut self) -> Result<Rank, PgnParseError> {
        match dbg!(self.tokens.peek()) {
            Some(Token::Number(number)) => match self.tokens.peek() {
                // This means that it was a move number and not a rank
                Some(Token::Number(..)) | Some(Token::Dot) => {
                    Err(PgnParseError::syntax(self.cursor))
                }
                None => Err(PgnParseError::unexpected_eof(self.cursor)),
                _ => {
                    let rank =
                        Rank::try_from(number).map_err(|_| PgnParseError::syntax(self.cursor))?;
                    self.consume();
                    Ok(rank)
                }
            },
            None => Err(PgnParseError::unexpected_eof(self.cursor)),
            _ => Err(PgnParseError::syntax(self.cursor)),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct PgnParseError {
    index: usize,
    message: String,
}

impl Display for PgnParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl PgnParseError {
    fn unexpected_eof(index: usize) -> Self {
        Self {
            index,
            message: format!("Unexpected end of file at index:{}", index),
        }
    }

    fn syntax(index: usize) -> Self {
        Self {
            index,
            message: format!("Syntax error, at index: {}", index),
        }
    }
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    let mut chars = input.char_indices().peekable();

    while let Some((idx, char)) = chars.next() {
        let token = match char {
            '/' => Token::Slash,
            '*' => Token::Star,
            ' ' | '\n' => continue,
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
enum Token {
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
    CastleShort,
    CastleLong,
    Invalid,
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum Nag {
    Good,
    Excellent,
    Interesting,
    Blunder,
    Poor,
    Dubious,
}

#[cfg(test)]
mod test {

    use super::{PgnParseError, PgnParser, Token, *};

    #[test]
    fn test_white_move_number() {
        let tokens = [Token::Number(1), Token::Dot];
        let mut parser = PgnParser::new(tokens.iter());
        assert_eq!(
            parser.move_number(),
            Ok(MoveNumber::WhiteMoveNumber("1".to_string()))
        );
    }

    #[test]
    fn test_black_move_number() {
        let tokens = [Token::Number(1), Token::Dot, Token::Dot, Token::Dot];
        let mut parser = PgnParser::new(tokens.iter());
        assert_eq!(
            parser.move_number(),
            Ok(MoveNumber::BlackMoveNumber("1".to_string()))
        );
    }

    #[test]
    fn test_black_move_number_without_dots() {
        let tokens = [Token::Number(1), Token::Number(1), Token::File('e')];
        let mut parser = PgnParser::new(tokens.iter());
        assert_eq!(
            parser.move_number(),
            Ok(MoveNumber::BlackMoveNumber("11".to_string()))
        );
    }

    #[test]
    fn test_multi_digit_move_number() {
        let tokens = [
            Token::Number(1),
            Token::Number(0),
            Token::Dot,
            Token::Dot,
            Token::Dot,
        ];
        let mut parser = PgnParser::new(tokens.iter());
        assert_eq!(
            parser.move_number(),
            Ok(MoveNumber::BlackMoveNumber("10".to_string()))
        );
    }

    #[test]
    fn test_move_text() {
        let tokens = tokenize("1.d4");
        let res = PgnParser::new(tokens.iter()).parse().unwrap();

        assert_eq!(res, MoveTree::new());
    }
    #[test]
    fn test_syntax_error_incomplete_dots() {
        let tokens = [Token::Number(1), Token::Dot, Token::Dot];
        let mut parser = PgnParser::new(tokens.iter());
        assert!(matches!(
            parser.move_number(),
            Err(PgnParseError { index: 0, .. })
        ));
    }

    #[test]
    fn test_unexpected_eof() {
        let tokens = [];
        let mut parser = PgnParser::new(tokens.iter());
        assert!(matches!(
            parser.move_number(),
            Err(PgnParseError { index: 0, .. })
        ));
    }

    #[test]
    fn test_syntax_error_no_number() {
        let tokens = [Token::File('a')];
        let mut parser = PgnParser::new(tokens.iter());
        assert!(matches!(
            parser.move_number(),
            Err(PgnParseError { index: 0, .. })
        ));
    }

    // #[test]
    // fn parses_nested_variations() {
    //     let tokens = tokenize("1. d4 ( 1. e4 e5 (2... Nf6 3. Nh3) ) d5 2.Nf3 (2. a4) 1-0");
    //     dbg!(tokens.clone());
    //     let res = PgnParser::new(tokens.iter()).parse().unwrap();
    //
    //     assert_eq!(res, MoveTree::new());
    // }
}
