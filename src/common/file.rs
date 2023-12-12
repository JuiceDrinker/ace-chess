use std::str::FromStr;

use crate::error::Error;

/// Describe a file (column) on a chess board.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[repr(u8)]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

/// Numbers of [`File`].
pub const NUM_FILES: usize = super::BOARD_SIZE.1 as usize;

/// Enumerate all files.
pub const ALL_FILES: [File; NUM_FILES] = [
    File::A,
    File::B,
    File::C,
    File::D,
    File::E,
    File::F,
    File::G,
    File::H,
];

impl File {
    /// Gets a [`File`] from an integer index.
    ///
    /// > **Note**: If index is not in the range 0..=7, wrap around.
    #[inline]
    pub fn new(index: usize) -> Self {
        ALL_FILES[index % NUM_FILES]
    }

    /// Convert this [`File`] into a [`usize`].
    #[inline]
    pub fn as_index(self) -> usize {
        self as usize
    }

    /// Go one file to the left.
    ///
    /// > **Note**: If impossible, wrap around.
    #[inline]
    pub fn left(self) -> Self {
        File::new(self.as_index().wrapping_sub(1))
    }

    /// Go one file to the right.
    ///
    /// > **Note**: If impossible, wrap around.
    #[inline]
    pub fn right(self) -> Self {
        File::new(self.as_index() + 1)
    }

    /// Distance between two [`File`].
    #[inline]
    pub fn distance(self, other: File) -> u32 {
        self.as_index().abs_diff(other.as_index()) as u32
    }

    /// Verify if the [`File`] is between two other (i.e. lower <= self <= upper).
    ///
    /// Assume that `lower_bound` <= `upper_bound`.
    #[inline]
    pub fn between(self, lower_bound: File, upper_bound: File) -> bool {
        lower_bound <= self && self <= upper_bound
    }

    pub fn as_str(&self) -> &str {
        match &self {
            File::A => "a",
            File::B => "b",
            File::C => "c",
            File::D => "d",
            File::E => "e",
            File::F => "f",
            File::G => "g",
            File::H => "h",
        }
    }
}

impl FromStr for File {
    type Err = Error;

    /// Only lowercase from a to h (inclusive).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(Error::InvalidFile);
        }
        match s.chars().next().unwrap() {
            'a' | '1' => Ok(File::A),
            'b' | '2' => Ok(File::B),
            'c' | '3' => Ok(File::C),
            'd' | '4' => Ok(File::D),
            'e' | '5' => Ok(File::E),
            'f' | '6' => Ok(File::F),
            'g' | '7' => Ok(File::G),
            'h' | '8' => Ok(File::H),
            _ => Err(Error::InvalidFile),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_index() {
        assert_eq!(File::A.as_index(), 0);
        assert_eq!(File::B.as_index(), 1);
        assert_eq!(File::C.as_index(), 2);
        assert_eq!(File::D.as_index(), 3);
        assert_eq!(File::E.as_index(), 4);
        assert_eq!(File::F.as_index(), 5);
        assert_eq!(File::G.as_index(), 6);
        assert_eq!(File::H.as_index(), 7);
    }

    #[test]
    fn right() {
        assert_eq!(File::A.right(), File::B);
        assert_eq!(File::B.right(), File::C);
        assert_eq!(File::C.right(), File::D);
        assert_eq!(File::D.right(), File::E);
        assert_eq!(File::E.right(), File::F);
        assert_eq!(File::F.right(), File::G);
        assert_eq!(File::G.right(), File::H);
        assert_eq!(File::H.right(), File::A);
    }

    #[test]
    fn left() {
        assert_eq!(File::A.left(), File::H);
        assert_eq!(File::B.left(), File::A);
        assert_eq!(File::C.left(), File::B);
        assert_eq!(File::D.left(), File::C);
        assert_eq!(File::E.left(), File::D);
        assert_eq!(File::F.left(), File::E);
        assert_eq!(File::G.left(), File::F);
        assert_eq!(File::H.left(), File::G);
    }

    #[test]
    fn distance() {
        assert_eq!(File::A.distance(File::A), 0);
        assert_eq!(File::A.distance(File::D), 3);
        assert_eq!(File::A.distance(File::H), 7);
    }

    #[test]
    fn between() {
        // expect true
        assert!(File::A.between(File::A, File::H));
        assert!(File::H.between(File::A, File::H));
        assert!(File::A.between(File::A, File::A));
        // expect false
        assert!(!File::A.between(File::B, File::H));
        assert!(!File::H.between(File::A, File::G));
        assert!(!File::B.between(File::C, File::A));
    }

    #[test]
    fn from_str() {
        assert_eq!(File::from_str("a"), Ok(File::A));
        assert_eq!(File::from_str("b"), Ok(File::B));
        assert_eq!(File::from_str("c"), Ok(File::C));
        assert_eq!(File::from_str("d"), Ok(File::D));
        assert_eq!(File::from_str("e"), Ok(File::E));
        assert_eq!(File::from_str("f"), Ok(File::F));
        assert_eq!(File::from_str("g"), Ok(File::G));
        assert_eq!(File::from_str("h"), Ok(File::H));
    }

    #[test]
    fn from_str_error() {
        assert_eq!(File::from_str(""), Err(Error::InvalidFile));
        assert_eq!(File::from_str(" a"), Err(Error::InvalidFile));
        assert_eq!(File::from_str("A"), Err(Error::InvalidFile));
    }
}
