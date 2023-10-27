use std::sync::{Arc, RwLock};

use crate::common::{
    board::Board, castle_rights::CastleRights, color::Color, piece::Piece, r#move::Move,
    rank::Rank, square::Square,
};

pub struct Writer {
    board: Arc<RwLock<Board>>,
}

impl Writer {
    pub fn new(board: Arc<RwLock<Board>>) -> Self {
        Writer { board }
    }
    pub fn read_board(&self) -> std::sync::RwLockReadGuard<'_, Board> {
        self.board.read().unwrap()
    }

    pub fn write_to_board(&self) -> std::sync::RwLockWriteGuard<'_, Board> {
        self.board.write().unwrap()
    }
    pub fn update_board(&self, m: Move) {
        let piece_from = self.read_board().piece_on(&m.from).unwrap();
        let side = self.read_board().side_to_move();
        let mut new_en_passant = false;
        let reset_halfmove = self.board.read().unwrap().piece_on_is(m.from, Piece::Pawn)
            || self.board.read().unwrap().is_occupied(m.to);

        match piece_from {
            // Pawn: En Passant, promotion
            Piece::Pawn => {
                self.write_to_board()[m.from] = None;
                self.write_to_board()[m.to] = Some((Piece::Pawn, side));
                // if En Passant: capture the pawn
                if self.read_board().en_passant() == Some(m.to) {
                    match side {
                        Color::White => self.write_to_board()[m.to.down()] = None,
                        Color::Black => self.write_to_board()[m.to.up()] = None,
                    }
                }
                // Set self.en_passant
                if m.distance() == 2 {
                    if self.read_board().on_is(m.to.left(), (Piece::Pawn, !side))
                        || self.read_board().on_is(m.to.right(), (Piece::Pawn, !side))
                    {
                        self.write_to_board().en_passant = Some(m.to.backward(side));
                        new_en_passant = true;
                    }
                } else {
                    self.write_to_board().en_passant = None;
                }

                // Promotion
                if m.to.rank_for(side) == Rank::Eighth {
                    self.write_to_board()[m.to] = Some((Piece::Queen, side));
                }
            }
            // King: Castle
            Piece::King => {
                if m.from.distance(m.to) == 2 {
                    if self.read_board().castle_rights(side).has_kingside()
                        && m.from.file() < m.to.file()
                    {
                        // if is a Castle - Kingside
                        self.write_to_board()[m.from] = None;
                        self.write_to_board()[m.to] = Some((Piece::King, side));
                        self.write_to_board()[m.to.right()] = None;
                        self.write_to_board()[m.to.left()] = Some((Piece::Rook, side));
                    } else if self.read_board().castle_rights(side).has_queenside()
                        && m.to.file() < m.from.file()
                    {
                        // if is a Castle - Queenside
                        self.write_to_board()[m.from] = None;
                        self.write_to_board()[m.to] = Some((Piece::King, side));
                        self.write_to_board()[m.to.left().left()] = None;
                        self.write_to_board()[m.to.right()] = Some((Piece::Rook, side));
                    } else {
                        panic!(
                            "Error::InvalidMove: Board: {}, invalid_move: {}",
                            self.read_board(),
                            m
                        );
                    }
                } else {
                    // normal move
                    self.write_to_board()[m.from] = None;
                    self.write_to_board()[m.to] = Some((Piece::King, side));
                }

                // If the king move he lost both CastleRights
                self.write_to_board()
                    .remove_castle_rights(side, CastleRights::Both);
            }
            // Rook: Castle
            Piece::Rook => {
                // remove CastleRights
                match m.from {
                    Square::A1 | Square::A8 => self
                        .write_to_board()
                        .remove_castle_rights(side, CastleRights::QueenSide),
                    Square::H1 | Square::H8 => self
                        .write_to_board()
                        .remove_castle_rights(side, CastleRights::KingSide),
                    _ => {}
                }
                self.write_to_board()[m.from] = None;
                self.write_to_board()[m.to] = Some((Piece::Rook, side));
            }
            _ => {
                self.write_to_board()[m.from] = None;
                self.write_to_board()[m.to] = Some((piece_from, side));
            }
        }

        self.write_to_board().side_to_move = !self.read_board().side_to_move;
        if !new_en_passant {
            self.write_to_board().en_passant = None;
        }
        self.write_to_board().halfmoves += 1;
        if reset_halfmove {
            self.write_to_board().halfmoves = 0;
        }
        if self.read_board().side_to_move == Color::White {
            self.write_to_board().fullmoves += 1;
        }
        dbg!(&self.board);
    }
}
