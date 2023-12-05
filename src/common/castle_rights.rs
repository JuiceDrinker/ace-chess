use super::color::Color;

/// What castle rights does a particular player have?
///
/// > rule: <https://en.wikipedia.org/wiki/Castling>
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Eq)]
pub enum CastleRights {
    /// No right to Castle.
    NoRights,
    /// Right to Castle only on King side (little Castle).
    KingSide,
    /// Right to Castle only on Queen side (big Castle).
    QueenSide,
    /// Right to Castle in both side (little and big Castle).
    Both,
}

impl CastleRights {
    /// Convert [`usize`] to [`CastleRights`].
    ///
    /// # Panics
    ///
    /// Panic if index is not in range 0..=3.
    pub fn from_index(index: usize) -> CastleRights {
        match index {
            0 => CastleRights::NoRights,
            1 => CastleRights::KingSide,
            2 => CastleRights::QueenSide,
            3 => CastleRights::Both,
            e => panic!("IndexError for CastleRights: {e}"),
        }
    }

    /// Convert [`CastleRights`] to [`usize`].
    pub fn as_index(self) -> usize {
        self as usize
    }

    /// Check the castling on the king side.
    pub fn has_kingside(self) -> bool {
        self.as_index() & 1 == 1
    }

    /// Check the castling on the queen side.
    pub fn has_queenside(self) -> bool {
        self.as_index() & 2 == 2
    }

    /// Convert the castle rights to an FEN compatible [`String`].
    ///
    /// ```
    /// use chess::{CastleRights, Color};
    ///
    /// assert_eq!(CastleRights::NoRights.to_string(Color::Black), "");
    /// assert_eq!(CastleRights::KingSide.to_string(Color::White), "K");
    /// assert_eq!(CastleRights::QueenSide.to_string(Color::Black), "q");
    /// assert_eq!(CastleRights::Both.to_string(Color::White), "KQ");
    /// ```
    pub fn as_string(self, color: Color) -> String {
        let result = match self {
            CastleRights::NoRights => "",
            CastleRights::KingSide => "k",
            CastleRights::QueenSide => "q",
            CastleRights::Both => "kq",
        };

        match color {
            Color::White => result.to_uppercase(),
            Color::Black => result.to_string(),
        }
    }
}
