use crate::error::Error;

pub const NUM_COLORS: usize = 2;
pub const BOARD_SIZE: (i16, i16) = (8, 8);

pub type Result<T> = anyhow::Result<T, Error>;
