use crate::common::{
    board::Board, castle_rights::CastleRights, color::Color, piece::Piece, r#move::Move,
    rank::Rank, square::Square,
};

pub fn update_board(m: Move, mut board: Board) -> Board {
    let piece_from = board.piece_on(&m.from).unwrap();
    let side = board.side_to_move();
    let mut new_en_passant = false;
    let reset_halfmove = board.piece_on_is(m.from, Piece::Pawn) || board.is_occupied(m.to);

    match piece_from {
        // Pawn: En Passant, promotion
        Piece::Pawn => {
            board[m.from] = None;
            board[m.to] = Some((Piece::Pawn, side));
            // if En Passant: capture the pawn
            if board.en_passant() == Some(m.to) {
                match side {
                    Color::White => board[m.to.down()] = None,
                    Color::Black => board[m.to.up()] = None,
                }
            }
            // Set en_passant
            if m.distance() == 2 {
                if board.on_is(m.to.left(), (Piece::Pawn, !side))
                    || board.on_is(m.to.right(), (Piece::Pawn, !side))
                {
                    board.en_passant = Some(m.to.backward(side));
                    new_en_passant = true;
                }
            } else {
                board.en_passant = None;
            }

            // Promotion
            if m.to.rank_for(side) == Rank::Eighth {
                board[m.to] = Some((Piece::Queen, side));
            }
        }
        // King: Castle
        Piece::King => {
            if m.from.distance(m.to) == 2 {
                if board.castle_rights(side).has_kingside() && m.from.file() < m.to.file() {
                    // if is a Castle - Kingside
                    board[m.from] = None;
                    board[m.to] = Some((Piece::King, side));
                    board[m.to.right()] = None;
                    board[m.to.left()] = Some((Piece::Rook, side));
                } else if board.castle_rights(side).has_queenside() && m.to.file() < m.from.file() {
                    // if is a Castle - Queenside
                    board[m.from] = None;
                    board[m.to] = Some((Piece::King, side));
                    board[m.to.left().left()] = None;
                    board[m.to.right()] = Some((Piece::Rook, side));
                } else {
                    panic!("Error::InvalidMove: Board: {}, invalid_move: {}", board, m);
                }
            } else {
                // normal move
                board[m.from] = None;
                board[m.to] = Some((Piece::King, side));
            }

            // If the king move he lost both CastleRights
            board.remove_castle_rights(side, CastleRights::Both);
        }
        // Rook: Castle
        Piece::Rook => {
            // remove CastleRights
            match m.from {
                Square::A1 | Square::A8 => {
                    board.remove_castle_rights(side, CastleRights::QueenSide)
                }
                Square::H1 | Square::H8 => board.remove_castle_rights(side, CastleRights::KingSide),
                _ => {}
            }
            board[m.from] = None;
            board[m.to] = Some((Piece::Rook, side));
        }
        _ => {
            board[m.from] = None;
            board[m.to] = Some((piece_from, side));
        }
    }

    board.side_to_move = !board.side_to_move;
    if !new_en_passant {
        board.en_passant = None;
    }
    board.halfmoves += 1;
    if reset_halfmove {
        board.halfmoves = 0;
    }
    if board.side_to_move == Color::White {
        board.fullmoves += 1;
    }
    board
}
