use std::fmt::Display;

#[derive(PartialEq, Eq, Debug)]
pub struct PgnParseError {
    pub index: usize,
    pub message: String,
}

impl Display for PgnParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl PgnParseError {
    pub fn unexpected_eof(index: usize) -> Self {
        Self {
            index,
            message: format!("Unexpected end of file at index: {}", index),
        }
    }

    pub fn syntax(index: usize, custom_message: &str) -> Self {
        Self {
            index,
            message: format!("Syntax error at index {}: {}", index, custom_message),
        }
    }

    pub fn expression_parsing_error(index: usize) -> Self {
        Self::syntax(
            index,
            "Failed to parse expression. Expected move text or variation.",
        )
    }

    pub fn variation_parsing_error(index: usize) -> Self {
        Self::syntax(
            index,
            "Failed to parse variation. Expected '(' followed by one or more moves.",
        )
    }

    pub fn result_parsing_error(index: usize) -> Self {
        Self::syntax(
            index,
            "Failed to parse game result. Expected '1-0', '0-1', '1/2-1/2', or '*'.",
        )
    }

    pub fn comment_parsing_error(index: usize) -> Self {
        Self::syntax(
            index,
            "Failed to parse comment. Expected '{' followed by comment text and '}'.",
        )
    }

    pub fn checkmate_parsing_error(index: usize) -> Self {
        Self::syntax(index, "Failed to parse checkmate. Expected '#'.")
    }

    pub fn check_parsing_error(index: usize) -> Self {
        Self::syntax(index, "Failed to parse check. Expected '+'.")
    }

    pub fn dot_parsing_error(index: usize) -> Self {
        Self::syntax(index, "Failed to parse dot. Expected '.'.")
    }

    pub fn move_text_parsing_error(index: usize) -> Self {
        Self::syntax(
            index,
            "Failed to parse move text. Expected move number followed by a move, or just a move.",
        )
    }

    pub fn move_number_parsing_error(index: usize) -> Self {
        Self::syntax(
            index,
            "Failed to parse move number. Expected number followed by '.', '...', or nothing.",
        )
    }

    pub fn move_parsing_error(index: usize) -> Self {
        Self::syntax(
            index,
            "Failed to parse move. Expected piece move, pawn move, or castling.",
        )
    }

    pub fn pawn_move_parsing_error(index: usize) -> Self {
        Self::syntax(index, "Failed to parse pawn move. Expected file and rank, possibly with capture and/or promotion.")
    }

    pub fn equals_parsing_error(index: usize) -> Self {
        Self::syntax(
            index,
            "Failed to parse equals sign. Expected '=' for pawn promotion.",
        )
    }

    pub fn castle_parsing_error(index: usize) -> Self {
        Self::syntax(
            index,
            "Failed to parse castling. Expected 'O-O' or 'O-O-O'.",
        )
    }

    pub fn piece_move_parsing_error(index: usize) -> Self {
        Self::syntax(index, "Failed to parse piece move. Expected piece symbol followed by destination, possibly with disambiguation.")
    }

    pub fn piece_capture_parsing_error(index: usize) -> Self {
        Self::syntax(index, "Failed to parse piece capture. Expected piece symbol, possibly file or rank, 'x', and destination.")
    }

    pub fn piece_parsing_error(index: usize) -> Self {
        Self::syntax(
            index,
            "Failed to parse piece. Expected 'N', 'B', 'R', 'Q', or 'K'.",
        )
    }

    pub fn captures_parsing_error(index: usize) -> Self {
        Self::syntax(index, "Failed to parse capture. Expected 'x'.")
    }

    pub fn file_parsing_error(index: usize) -> Self {
        Self::syntax(index, "Failed to parse file. Expected 'a' through 'h'.")
    }

    pub fn rank_parsing_error(index: usize) -> Self {
        Self::syntax(index, "Failed to parse rank. Expected '1' through '8'.")
    }
}
