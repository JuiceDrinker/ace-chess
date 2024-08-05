use std::str::FromStr;

use crate::error::Error;

use super::BOARD_SIZE;

/// Describe a rank (row) on a chess board.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[repr(u8)]
pub enum Rank {
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth,
    Seventh,
    Eighth,
}

/// Numbers of [`Rank`].
pub const NUM_RANKS: usize = BOARD_SIZE.1 as usize;

/// Enumerate all ranks.
pub const ALL_RANKS: [Rank; NUM_RANKS] = [
    Rank::First,
    Rank::Second,
    Rank::Third,
    Rank::Fourth,
    Rank::Fifth,
    Rank::Sixth,
    Rank::Seventh,
    Rank::Eighth,
];

impl Rank {
    /// Gets a [`Rank`] from an integer index.
    ///
    /// > **Note**: If index is not in the range 0..=7, wrap around.
    #[inline]
    pub fn new(index: usize) -> Self {
        ALL_RANKS[index % NUM_RANKS]
    }

    /// Convert this [`Rank`] into a [`usize`].
    #[inline]
    pub fn as_index(&self) -> usize {
        *self as usize
    }

    /// Go one rank up.
    ///
    /// > **Note**: If impossible, wrap around.
    #[inline]
    pub fn up(&self) -> Self {
        Rank::new(self.as_index() + 1)
    }

    /// Go one rank down.
    ///
    /// > **Note**: If impossible, wrap around.
    #[inline]
    pub fn down(&self) -> Self {
        let idx = self.as_index();
        match idx {
            0 => Rank::new(NUM_RANKS - 1),
            _ => Rank::new(idx - 1),
        }
    }

    /// Distance between two [`Rank`].
    #[inline]
    pub fn distance(&self, other: Rank) -> u32 {
        self.as_index().abs_diff(other.as_index()) as u32
    }

    /// Verify if the [`Rank`] is between two other (i.e. lower <= self <= upper).
    ///
    /// Assume that lower_bound <= upper_bound.
    #[inline]
    pub fn between(&self, lower_bound: Rank, upper_bound: Rank) -> bool {
        lower_bound <= *self && *self <= upper_bound
    }
}

impl FromStr for Rank {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(Error::InvalidRank);
        }
        match s.chars().next().unwrap() {
            '1' => Ok(Rank::First),
            '2' => Ok(Rank::Second),
            '3' => Ok(Rank::Third),
            '4' => Ok(Rank::Fourth),
            '5' => Ok(Rank::Fifth),
            '6' => Ok(Rank::Sixth),
            '7' => Ok(Rank::Seventh),
            '8' => Ok(Rank::Eighth),
            _ => Err(Error::InvalidRank),
        }
    }
}

impl TryFrom<&u32> for Rank {
    type Error = Error;

    fn try_from(value: &u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Rank::First),
            2 => Ok(Rank::Second),
            3 => Ok(Rank::Third),
            4 => Ok(Rank::Fourth),
            5 => Ok(Rank::Fifth),
            6 => Ok(Rank::Sixth),
            7 => Ok(Rank::Seventh),
            8 => Ok(Rank::Eighth),
            _ => Err(Error::InvalidRank),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_index() {
        assert_eq!(Rank::First.as_index(), 0);
        assert_eq!(Rank::Second.as_index(), 1);
        assert_eq!(Rank::Third.as_index(), 2);
        assert_eq!(Rank::Fourth.as_index(), 3);
        assert_eq!(Rank::Fifth.as_index(), 4);
        assert_eq!(Rank::Sixth.as_index(), 5);
        assert_eq!(Rank::Seventh.as_index(), 6);
        assert_eq!(Rank::Eighth.as_index(), 7);
    }

    #[test]
    fn up() {
        assert_eq!(Rank::First.up(), Rank::Second);
        assert_eq!(Rank::Second.up(), Rank::Third);
        assert_eq!(Rank::Third.up(), Rank::Fourth);
        assert_eq!(Rank::Fourth.up(), Rank::Fifth);
        assert_eq!(Rank::Fifth.up(), Rank::Sixth);
        assert_eq!(Rank::Sixth.up(), Rank::Seventh);
        assert_eq!(Rank::Seventh.up(), Rank::Eighth);
        assert_eq!(Rank::Eighth.up(), Rank::First);
    }

    #[test]
    fn down() {
        assert_eq!(Rank::First.down(), Rank::Eighth);
        assert_eq!(Rank::Second.down(), Rank::First);
        assert_eq!(Rank::Third.down(), Rank::Second);
        assert_eq!(Rank::Fourth.down(), Rank::Third);
        assert_eq!(Rank::Fifth.down(), Rank::Fourth);
        assert_eq!(Rank::Sixth.down(), Rank::Fifth);
        assert_eq!(Rank::Seventh.down(), Rank::Sixth);
        assert_eq!(Rank::Eighth.down(), Rank::Seventh);
    }

    #[test]
    fn distance() {
        assert_eq!(Rank::First.distance(Rank::First), 0);
        assert_eq!(Rank::First.distance(Rank::Fourth), 3);
        assert_eq!(Rank::First.distance(Rank::Eighth), 7);
    }

    #[test]
    fn between() {
        // expect true
        assert!(Rank::First.between(Rank::First, Rank::Eighth));
        assert!(Rank::Eighth.between(Rank::First, Rank::Eighth));
        assert!(Rank::First.between(Rank::First, Rank::First));
        // expect false
        assert!(!Rank::First.between(Rank::Second, Rank::Eighth));
        assert!(!Rank::Eighth.between(Rank::First, Rank::Seventh));
        assert!(!Rank::Second.between(Rank::Third, Rank::First));
    }

    #[test]
    fn from_str() {
        assert_eq!(Rank::from_str("1"), Ok(Rank::First));
        assert_eq!(Rank::from_str("2"), Ok(Rank::Second));
        assert_eq!(Rank::from_str("3"), Ok(Rank::Third));
        assert_eq!(Rank::from_str("4"), Ok(Rank::Fourth));
        assert_eq!(Rank::from_str("5"), Ok(Rank::Fifth));
        assert_eq!(Rank::from_str("6"), Ok(Rank::Sixth));
        assert_eq!(Rank::from_str("7"), Ok(Rank::Seventh));
        assert_eq!(Rank::from_str("8"), Ok(Rank::Eighth));
    }

    #[test]
    fn from_str_error() {
        assert_eq!(Rank::from_str(""), Err(Error::InvalidRank));
        assert_eq!(Rank::from_str(" 1"), Err(Error::InvalidRank));
        assert_eq!(Rank::from_str("second"), Err(Error::InvalidRank));
    }
}
