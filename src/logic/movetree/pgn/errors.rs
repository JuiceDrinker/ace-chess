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
            message: format!("Unexpected end of file at index:{}", index),
        }
    }

    pub fn syntax(index: usize) -> Self {
        Self {
            index,
            message: format!("Syntax error, at index: {}", index),
        }
    }
}
