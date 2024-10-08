use std::{
    fmt::{Debug, Display},
    iter::Peekable,
    slice::Iter,
    str::FromStr,
};

use crate::{
    common::{board::Board, color::Color, file::File, piece::Piece, rank::Rank, square::Square},
    error::Error,
    logic::movetree::{
        treenode::{CMove, CMoveKind, CResult, CastleSide, Fen, MoveDetails, TreeNode},
        MoveTree,
    },
};

use super::{errors::PgnParseError, lexer::Token};

pub const STARTING_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

// Grammar
// R: 1 … 8               # Rank
// F: a … h               # File
// P: N, B, R, Q, K       # Piece
// PM: PFFR | PRFR | PFR | PC # Piece Move
// PC: P'x'FR | PFxFR | PRxFR # Piece capture
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
    move_tree: MoveTree,
    tokens: Peekable<Iter<'a, Token>>,
    cursor: usize,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Move(CMove),
    Variation(Vec<Expression>),
}

#[derive(Debug, PartialEq, Eq)]
enum MoveNumber {
    WhiteMoveNumber(usize),
    BlackMoveNumber(usize),
}

impl Display for MoveNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            MoveNumber::WhiteMoveNumber(number) => format!("{}. ", number),
            MoveNumber::BlackMoveNumber(number) => format!("{}... ", number),
        };

        write!(f, "{value}")
    }
}

impl<'a> PgnParser<'a> {
    pub fn new(tokens: Iter<'a, Token>) -> Self {
        Self {
            move_tree: MoveTree::default(),
            tokens: tokens.peekable(),
            cursor: 0,
        }
    }

    pub fn parse(&mut self) -> Result<MoveTree, PgnParseError> {
        // let mut move_tree = MoveTree::new();
        let mut current = self.move_tree.game_start();

        while let Ok(expression) = self.expression() {
            let node = self.add_expression_to_tree(expression, current);
            current = node;
        }

        let result = self.result()?;
        let new_node = self.move_tree.tree.new_node(TreeNode::Result(result));
        current.append(new_node, &mut self.move_tree.tree);

        Ok(self.move_tree.clone())
    }

    fn add_move_to_tree(&mut self, cmove: CMove, parent: indextree::NodeId) -> indextree::NodeId {
        // Not convinced this works but tests pass...
        if let Ok(fen) = generate_next_fen(&self.get_last_fen(parent), &cmove) {
            let new_node = self
                .move_tree
                .tree
                .new_node(TreeNode::Move(fen.clone(), cmove));
            parent.append(new_node, &mut self.move_tree.tree);
            new_node
        } else {
            match self.move_tree.tree[parent].parent() {
                Some(grandparent) => {
                    let fen = self.get_last_fen(grandparent);
                    let new_node = self
                        .move_tree
                        .tree
                        .new_node(TreeNode::Move(fen.clone(), cmove));
                    parent.append(new_node, &mut self.move_tree.tree);
                    new_node
                }
                None => panic!("Could not add move: {} to tree", cmove.to_san()),
            }
        }
    }

    fn add_expression_to_tree(
        &mut self,
        expression: Expression,
        parent: indextree::NodeId,
    ) -> indextree::NodeId {
        match expression {
            Expression::Move(cmove) => self.add_move_to_tree(cmove, parent),
            Expression::Variation(expressions) => self.add_variation_to_tree(expressions, parent),
        }
    }

    fn add_variation_to_tree(
        &mut self,
        expressions: Vec<Expression>,
        parent: indextree::NodeId,
    ) -> indextree::NodeId {
        let grand_parent = self.move_tree.tree[parent].parent().unwrap();
        let start_variation = self.move_tree.tree.new_node(TreeNode::StartVariation);
        grand_parent.append(start_variation, &mut self.move_tree.tree);

        let mut var_current = start_variation;

        for expression in expressions {
            let new_node = self.add_expression_to_tree(expression, var_current);
            var_current = if let TreeNode::EndVariation = self.move_tree.tree[new_node].get() {
                parent
            } else {
                new_node
            }
        }

        let end_variation = self.move_tree.tree.new_node(TreeNode::EndVariation);
        var_current.append(end_variation, &mut self.move_tree.tree);

        end_variation
    }

    // If given node has FEN, return it
    // else traverse up tree till you find one
    fn get_last_fen(&self, node: indextree::NodeId) -> Fen {
        if let TreeNode::Move(fen, _) = self.move_tree.tree[node].get() {
            return fen.clone();
        } else {
            let mut current = node;
            while let Some(parent) = self.move_tree.tree.get(current).unwrap().parent() {
                if let TreeNode::Move(fen, _) = self.move_tree.tree[parent].get() {
                    return fen.clone();
                }
                current = parent;
            }
        }
        // If we can't find a parent move, return the starting position
        STARTING_POSITION_FEN.to_string()
    }

    // E: MT C? | V | E E    # Element (allows for comments and variations between moves)
    fn expression(&mut self) -> Result<Expression, PgnParseError> {
        if let Ok(mut move_text) = self.move_text() {
            if let Ok(comment) = self.comment() {
                move_text.comment = Some(comment);
                Ok(Expression::Move(move_text))
            } else {
                Ok(Expression::Move(move_text))
            }
        } else if let Ok(variation) = self.variation() {
            Ok(Expression::Variation(variation))
        } else {
            return Err(PgnParseError::expression_parsing_error(self.cursor));
        }
    }

    fn variation(&mut self) -> Result<Vec<Expression>, PgnParseError> {
        if self.tokens.peek().is_none() {
            return Err(PgnParseError::unexpected_eof(self.cursor));
        };

        if let Some(Token::StartVariation) = self.tokens.peek() {
            self.consume();
            let mut expressions = vec![];
            while let Some(token) = self.tokens.peek() {
                if token == &&Token::EndVariation {
                    self.consume();
                    return Ok(expressions);
                }

                if let Ok(expression) = self.expression() {
                    expressions.push(expression)
                } else {
                    return Err(PgnParseError::variation_parsing_error(self.cursor));
                }
            }
        }

        Err(PgnParseError::variation_parsing_error(self.cursor))
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
                            Err(PgnParseError::result_parsing_error(self.cursor))
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
                        Err(PgnParseError::result_parsing_error(self.cursor))
                    }
                    _ => {
                        self.tokens = iter_save;
                        self.cursor = cursor_save;
                        Err(PgnParseError::result_parsing_error(self.cursor))
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
                        Err(PgnParseError::result_parsing_error(self.cursor))
                    }
                } else {
                    self.tokens = iter_save;
                    self.cursor = cursor_save;
                    Err(PgnParseError::result_parsing_error(self.cursor))
                }
            }
            Some(Token::Star) => {
                self.consume();
                Ok(CResult::NoResult)
            }
            Some(_) => Err(PgnParseError::result_parsing_error(self.cursor)),
            None => Err(PgnParseError::unexpected_eof(self.cursor)),
        }
    }
    fn comment(&mut self) -> Result<String, PgnParseError> {
        match self.tokens.peek() {
            Some(Token::Comment(comment)) => {
                self.consume();
                Ok(comment.to_string())
            }
            Some(_) => Err(PgnParseError::comment_parsing_error(self.cursor)),
            None => Err(PgnParseError::unexpected_eof(self.cursor)),
        }
    }

    fn checkmate(&mut self) -> Result<String, PgnParseError> {
        match self.tokens.peek() {
            Some(Token::Checkmate) => {
                self.consume();
                Ok("#".to_string())
            }
            Some(_) => Err(PgnParseError::checkmate_parsing_error(self.cursor)),
            None => Err(PgnParseError::unexpected_eof(self.cursor)),
        }
    }

    fn check(&mut self) -> Result<String, PgnParseError> {
        match self.tokens.peek() {
            Some(Token::Check) => {
                self.consume();
                Ok("+".to_string())
            }
            Some(_) => Err(PgnParseError::check_parsing_error(self.cursor)),
            None => Err(PgnParseError::unexpected_eof(self.cursor)),
        }
    }

    fn dot(&mut self) -> Result<String, PgnParseError> {
        match self.tokens.peek() {
            Some(Token::Dot) => {
                self.consume();
                Ok(".".to_string())
            }
            Some(_) => Err(PgnParseError::dot_parsing_error(self.cursor)),
            None => Err(PgnParseError::unexpected_eof(self.cursor)),
        }
    }

    // MT: M | MN D M | MN DDD M  # Move Text
    fn move_text(&mut self) -> Result<CMove, PgnParseError> {
        let iter_save = self.tokens.clone();
        let cursor_save = self.cursor;
        if let Ok(mut move_kind) = self.r#move() {
            move_kind.color = Color::Black;
            return Ok(move_kind);
        } else if let Ok(move_number) = self.move_number() {
            if let Ok(mut move_kind) = self.r#move() {
                return match move_number {
                    MoveNumber::WhiteMoveNumber(_) => Ok(move_kind),
                    MoveNumber::BlackMoveNumber(number) => {
                        move_kind.move_number = number;
                        move_kind.color = Color::Black;
                        Ok(move_kind)
                    }
                };
            }
            self.tokens = iter_save;
            self.cursor = cursor_save
        }
        Err(PgnParseError::move_text_parsing_error(self.cursor))
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
                                return Ok(MoveNumber::BlackMoveNumber(number.parse().unwrap()));
                            } else {
                                self.tokens = iter_save;
                                self.cursor = cursor_save;
                                return Err(PgnParseError::move_number_parsing_error(self.cursor));
                            }
                        } else {
                            return Ok(MoveNumber::WhiteMoveNumber(number.parse().unwrap()));
                        }
                    } else if let Some(Token::Number(n)) = self.tokens.peek() {
                        number.push_str(&n.to_string());
                        self.consume();
                    } else {
                        self.consume();
                        return Ok(MoveNumber::BlackMoveNumber(number.parse().unwrap()));
                    }
                }
            }
            Some(_) => Err(PgnParseError::move_number_parsing_error(self.cursor)),
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
            return Err(PgnParseError::move_parsing_error(self.cursor));
        };

        if self.check().is_ok() {
            return Ok(CMove {
                move_number: 0,
                kind: move_kind,
                check: true,
                checkmate: false,
                color: Color::White,
                comment: None,
            });
        } else if self.checkmate().is_ok() {
            return Ok(CMove {
                move_number: 0,
                kind: move_kind,
                check: false,
                checkmate: true,
                color: Color::White,
                comment: None,
            });
        }

        Ok(CMove {
            move_number: 0,
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
        let file = self.file()?;

        if let Ok(rank) = self.rank() {
            if self.equals().is_ok() {
                if let Ok(piece) = self.piece() {
                    let cmove = MoveDetails {
                        piece: Piece::Pawn,
                        dst_rank: rank,
                        dst_file: file,
                        captures: false,
                        src_rank: None,
                        src_file: None,
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
                    src_rank: None,
                    src_file: None,
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
                                src_rank: None,
                                src_file: Some(file),
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
                            src_rank: None,
                            src_file: Some(file),
                            promotion: None,
                        };
                        return Ok(CMoveKind::Regular(cmove));
                    }
                }
            }
        }

        self.tokens = iter_save;
        self.cursor = cursor_save;

        Err(PgnParseError::pawn_move_parsing_error(self.cursor))
    }

    fn equals(&mut self) -> Result<(), PgnParseError> {
        match self.tokens.peek() {
            Some(Token::Equals) => {
                self.consume();
                Ok(())
            }
            Some(_) => Err(PgnParseError::equals_parsing_error(self.cursor)),
            None => Err(PgnParseError::unexpected_eof(self.cursor)),
        }
    }

    fn castle(&mut self) -> Result<CMoveKind, PgnParseError> {
        match self.tokens.peek() {
            Some(Token::Number(0)) => {
                let iter_save = self.tokens.clone();
                let cursor_save = self.cursor;

                self.consume();
                if let Some(Token::Hyphen) = self.tokens.peek() {
                    self.consume();
                    if let Some(Token::Number(0)) = self.tokens.peek() {
                        self.consume();
                        if let Some(Token::Hyphen) = self.tokens.peek() {
                            self.consume();
                            if let Some(Token::Number(0)) = self.tokens.peek() {
                                return Ok(CMoveKind::Castles(CastleSide::Long));
                            }
                        }
                        return Ok(CMoveKind::Castles(CastleSide::Short));
                    }
                }

                self.tokens = iter_save;
                self.cursor = cursor_save;
                Err(PgnParseError::castle_parsing_error(self.cursor))
            }
            Some(_) => Err(PgnParseError::castle_parsing_error(self.cursor)),
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
            return Err(PgnParseError::piece_move_parsing_error(self.cursor));
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
                        src_rank: None,
                        src_file: Some(file),
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
                    src_rank: None,
                    src_file: None,
                    promotion: None,
                };
                return Ok(CMoveKind::Regular(cmove));
            }

            self.tokens = iter_save;
            self.cursor = cursor_save;
            return Err(PgnParseError::piece_move_parsing_error(self.cursor));
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
                        src_rank: Some(source_rank),
                        src_file: None,
                        promotion: None,
                    };
                    return Ok(CMoveKind::Regular(cmove));
                }
            }
            self.tokens = iter_save;
            self.cursor = cursor_save;
            return Err(PgnParseError::piece_move_parsing_error(self.cursor));
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
                        src_rank: None,
                        src_file: None,
                        promotion: None,
                    };
                    return Ok(CMoveKind::Regular(cmove));
                }
            }
            self.tokens = iter_save;
            self.cursor = cursor_save;
            return Err(PgnParseError::piece_capture_parsing_error(self.cursor));
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
                            src_rank: None,
                            src_file: Some(src_file),
                            promotion: None,
                        };
                        return Ok(CMoveKind::Regular(cmove));
                    }
                }
            }
            self.tokens = iter_save;
            self.cursor = cursor_save;
            return Err(PgnParseError::piece_capture_parsing_error(self.cursor));
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
                            src_rank: Some(from_rank),
                            src_file: None,
                            promotion: None,
                        };
                        return Ok(CMoveKind::Regular(cmove));
                    }
                }
            }
            self.tokens = iter_save;
            self.cursor = cursor_save;
            return Err(PgnParseError::piece_capture_parsing_error(self.cursor));
        }

        Err(PgnParseError::piece_capture_parsing_error(self.cursor))
    }

    // P: N, B, R, Q, K       # Piece
    fn piece(&mut self) -> Result<Piece, PgnParseError> {
        let piece = match self.tokens.peek() {
            Some(Token::Piece(piece)) => {
                self.consume();
                piece
            }
            None => return Err(PgnParseError::unexpected_eof(self.cursor)),
            _ => return Err(PgnParseError::piece_parsing_error(self.cursor)),
        };

        Ok(*piece)
    }

    fn captures(&mut self) -> Result<(), PgnParseError> {
        match self.tokens.peek() {
            Some(Token::Captures) => {
                self.consume();
                Ok(())
            }
            Some(_) => Err(PgnParseError::captures_parsing_error(self.cursor)),
            None => Err(PgnParseError::unexpected_eof(self.cursor)),
        }
    }

    // 'a' - 'h'
    fn file(&mut self) -> Result<File, PgnParseError> {
        match self.tokens.peek() {
            Some(Token::File(file)) => {
                let file = File::from_str(&file.to_string())
                    .map_err(|_| PgnParseError::file_parsing_error(self.cursor))?;
                self.consume();
                Ok(file)
            }

            Some(_) => Err(PgnParseError::file_parsing_error(self.cursor)),
            None => Err(PgnParseError::unexpected_eof(self.cursor)),
        }
    }

    // R: 1 … 8               # Rank
    fn rank(&mut self) -> Result<Rank, PgnParseError> {
        match self.tokens.peek() {
            Some(Token::Number(number)) => {
                // NOTE: Feel this is a bit dangerous incase we encounter a move number from 1-8
                // and parse it as rank but other grammar rule should fail meaning we dont eat this
                // token as a rank when it isn't
                self.consume();
                let rank = Rank::try_from(number)
                    .map_err(|_| PgnParseError::rank_parsing_error(self.cursor))?;
                Ok(rank)
            }
            Some(_) => Err(PgnParseError::rank_parsing_error(self.cursor)),
            None => Err(PgnParseError::unexpected_eof(self.cursor)),
        }
    }
}

pub fn generate_next_fen(current_fen: &str, cmove: &CMove) -> crate::Result<Fen> {
    // NOTE: Currently board struct only handles promotion to queen
    let mut board = Board::from_str(current_fen).expect("To be able to create a valid fen");

    match &cmove.kind {
        CMoveKind::Castles(side) => {
            let (from, to) = match (cmove.color, side) {
                (Color::White, CastleSide::Short) => (Square::E1, Square::G1),
                (Color::White, CastleSide::Long) => (Square::E1, Square::C1),
                (Color::Black, CastleSide::Short) => (Square::E8, Square::G8),
                (Color::Black, CastleSide::Long) => (Square::E8, Square::C8),
            };
            Ok(board
                .update(crate::common::r#move::Move { from, to })
                .to_string())
        }
        CMoveKind::Regular(details) => {
            let dest = Square::make_square(details.dst_file, details.dst_rank);
            let potential_source_squares = board.get_valid_moves_to(dest, details.piece);

            if potential_source_squares.len() == 1 {
                board.update(crate::common::r#move::Move {
                    from: potential_source_squares.into_iter().next().unwrap(),
                    to: dest,
                });

                Ok(board.to_string())
            } else {
                // Handle disambiguation
                let mut from_square = None;
                for square in potential_source_squares {
                    if (details.src_file.is_some() && square.file() == details.src_file.unwrap())
                        || (details.src_rank.is_some()
                            && square.rank() == details.src_rank.unwrap())
                    {
                        from_square = Some(square);
                        break;
                    }
                }

                if let Some(from) = from_square {
                    board.update(crate::common::r#move::Move { from, to: dest });
                } else {
                    return Err(Error::FenGeneration {
                        fen: board.to_string(),
                        cmove: cmove.clone(),
                    });
                }
                Ok(board.to_string())
            }
        }
    }
}
#[cfg(test)]
mod test {
    use crate::logic::movetree::pgn::lexer::{tokenize, Token};

    use super::{PgnParseError, PgnParser, *};

    #[test]
    fn next_fen() {
        let res = generate_next_fen(
            STARTING_POSITION_FEN,
            &CMove {
                kind: CMoveKind::Regular(MoveDetails {
                    piece: Piece::Pawn,
                    dst_rank: Rank::Fourth,
                    dst_file: File::D,
                    captures: false,
                    src_rank: None,
                    src_file: None,
                    promotion: None,
                }),
                move_number: 1,
                check: false,
                color: Color::White,
                checkmate: false,
                comment: None,
            },
        );
        assert_eq!(
            res,
            Ok("rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq - 0 1".to_string())
        );
    }
    #[test]
    fn test_white_move_number() {
        let tokens = [Token::Number(1), Token::Dot];
        let mut parser = PgnParser::new(tokens.iter());
        assert_eq!(parser.move_number(), Ok(MoveNumber::WhiteMoveNumber(1)));
    }

    #[test]
    fn test_black_move_number() {
        let tokens = [Token::Number(1), Token::Dot, Token::Dot, Token::Dot];
        let mut parser = PgnParser::new(tokens.iter());
        assert_eq!(parser.move_number(), Ok(MoveNumber::BlackMoveNumber(1)));
    }

    #[test]
    fn test_black_move_number_without_dots() {
        let tokens = [Token::Number(1), Token::Number(1), Token::File('e')];
        let mut parser = PgnParser::new(tokens.iter());
        assert_eq!(parser.move_number(), Ok(MoveNumber::BlackMoveNumber(11)));
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
        assert_eq!(parser.move_number(), Ok(MoveNumber::BlackMoveNumber(10)));
    }

    #[test]
    fn test_move_text() {
        let tokens = tokenize("1.d4");
        let res = PgnParser::new(tokens.iter()).move_text().unwrap();

        assert_eq!(
            res,
            CMove {
                kind: CMoveKind::Regular({
                    MoveDetails {
                        piece: Piece::Pawn,
                        dst_rank: Rank::Fourth,
                        dst_file: File::D,
                        captures: false,
                        src_rank: None,
                        src_file: None,
                        promotion: None,
                    }
                }),
                move_number: 0,
                check: false,
                color: Color::White,
                checkmate: false,
                comment: None,
            }
        );
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

    #[test]
    fn test_rank() {
        let tokens = tokenize("4");
        let res = PgnParser::new(tokens.iter()).rank().unwrap();

        assert_eq!(res, Rank::Fourth);
    }

    #[test]
    fn test_simple_pawn_move() {
        let tokens = tokenize("d4");
        let res = PgnParser::new(tokens.iter()).pawn_move().unwrap();

        assert_eq!(
            res,
            CMoveKind::Regular({
                MoveDetails {
                    piece: Piece::Pawn,
                    dst_rank: Rank::Fourth,
                    dst_file: File::D,
                    captures: false,
                    src_rank: None,
                    src_file: None,
                    promotion: None,
                }
            })
        );
    }

    #[test]
    fn test_variation() {
        let tokens = tokenize("( 1. e4 e5 )");
        let res = PgnParser::new(tokens.iter()).variation().unwrap();
        assert_eq!(res.len(), 2)
    }

    //
    // #[test]
    // fn debug() {
    //     let tokens = tokenize(
    //         "1. e4 e5 2. Nf3 Nc6 3. Bb5  3... a6
    //                  4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. c3 O-O 9. h3 Nb8 10. d4 Nbd7
    // 11. c4 c6 12. cxb5 axb5 13. Nc3 Bb7 14. Bg5 b4 15. Nb1 h6 16. Bh4 c5 17. dxe5
    //             Nxe4 18. Bxe7 Qxe7 19. exd6 Qf6 20. Nbd2 Nxd6 21. Nc4 Nxc4 22. Bxc4 Nb6
    //             23. Ne5 Rae8 24. Bxf7+ Rxf7 25. Nxf7 Rxe1+ 26. Qxe1 Kxf7 27. Qe3 Qg5 28. Qxg5
    //             hxg5 29. b3 Ke6 30. a3 Kd6 31. axb4 cxb4 32. Ra5 Nd5 33. f3 Bc8 34. Kf2 Bf5
    //             35. Ra7 g6 36. Ra6+ Kc5 37. Ke1 Nf4 38. g3 Nxh3 39. Kd2 Kb5 40. Rd6 Kc5 41. Ra6
    //             Nf2 42. g4 Bd3 43. Re6
    //     1/2-1/2",
    //     );
    //
    //     let pgn = PgnParser::new(tokens.iter()).parse();
    //     assert_eq!(pgn, Ok(MoveTree::new()));
    // }

    #[test]
    fn parses_variations_multiple() {
        let tokens = tokenize("1.d4 e5 (1... Na6) (1... Nh6) (1...Nf6) 0-1");
        let res = PgnParser::new(tokens.iter()).parse();

        assert!(res.is_ok());
    }

    #[test]
    fn parses_variations_nested() {
        let tokens = tokenize("1.d4 e5 (1... Nh6 2.e4 (e3)) (1...Nf6) 0-1");
        let res = PgnParser::new(tokens.iter()).parse();

        assert!(res.is_ok());
    }
    #[test]
    fn parses_variations() {
        let tokens = tokenize("1.d4 e5 (1... Nh6) (1...Nf6) 0-1");
        let res = PgnParser::new(tokens.iter()).parse();

        assert!(res.is_ok());
    }

    #[test]
    fn test_simple_game() {
        let tokens = tokenize("1.d4 1-0");
        let res = PgnParser::new(tokens.iter()).parse();
        assert!(res.is_ok())
    }
}
