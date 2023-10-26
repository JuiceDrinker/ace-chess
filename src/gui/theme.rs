use ggez::graphics::Color;

use crate::prelude::{NUM_COLORS, NUM_PIECES};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Theme {
    pub background_color: Color,
    pub board_color: [Color; NUM_COLORS],
    pub piece_path: [[&'static str; NUM_PIECES]; NUM_COLORS],
    pub valid_moves_color: Option<Color>,
    pub piece_pinned_color: Option<Color>,
    pub piece_pinned_path: Option<&'static str>,
    pub theme_icon_path: Option<&'static str>,
    pub font_path: &'static str,
    pub font_scale: f32,
}

impl Default for Theme {
    fn default() -> Self {
        THEME_DUST
    }
}

pub const THEME_DUST: Theme = Theme {
    background_color: Color::new(0.09, 0.09, 0.11, 1.0),
    board_color: [
        Color::new(0.7969, 0.7148, 0.6797, 1.0),
        Color::new(0.4375, 0.3984, 0.4648, 1.0),
    ],
    piece_path: [
        [
            "/images/pieces/white_pawn.png",
            "/images/pieces/white_knight.png",
            "/images/pieces/white_bishop.png",
            "/images/pieces/white_rook.png",
            "/images/pieces/white_queen.png",
            "/images/pieces/white_king.png",
        ],
        [
            "/images/pieces/black_pawn.png",
            "/images/pieces/black_knight.png",
            "/images/pieces/black_bishop.png",
            "/images/pieces/black_rook.png",
            "/images/pieces/black_queen.png",
            "/images/pieces/black_king.png",
        ],
    ],
    valid_moves_color: Some(Color::new(0.25, 0.75, 0.25, 0.5)),
    piece_pinned_color: Some(Color::new(0.75, 0.25, 0.25, 0.5)),
    piece_pinned_path: Some("/images/pin.png"),
    theme_icon_path: Some("/images/theme_icon_white.png"),
    font_path: "/fonts/LiberationMono-Regular.ttf",
    font_scale: 20.0,
};
