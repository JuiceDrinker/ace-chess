use std::fmt;

use ggez::{graphics, Context, GameResult};

use crate::event::Event;

use super::config::{BOARD_PX_SIZE, SIDE_SCREEN_PX_SIZE};

/// Indicate how align the text (GUI).
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub enum Align {
    Left,
    Right,
    #[default]
    Center,
}

/// A struct of button for interact with the GUI.
#[derive(Clone)]
pub struct Button {
    /// The id is not unique, it's just a name to identify it.
    pub id: &'static str,
    enable: bool,
    rect: graphics::Rect,
    image_path: Option<&'static str>,
    color: graphics::Color,
    text: &'static str,
    align: Align,
    on_click: Event,
}

impl Button {
    /// Create a new [`Button`].
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: &'static str,
        enable: bool,
        rect: graphics::Rect,
        color: graphics::Color,
        text: &'static str,
        align: Align,
        on_click: Event,
    ) -> Self {
        Button {
            id,
            enable,
            rect,
            image_path: None,
            color,
            text,
            align,
            on_click,
        }
    }

    /// Enable the button.
    pub fn enable(&mut self) {
        self.enable = true;
    }

    /// Disable the button.
    pub fn disable(&mut self) {
        self.enable = false;
    }

    /// Draw the image at the given path rather than a rectangle.
    pub fn set_image(&mut self, path: Option<&'static str>) -> Self {
        self.image_path = path;
        self.clone()
    }

    /// Verify if a coordinate is in the button.
    pub fn contains(&self, x: f32, y: f32) -> bool {
        self.rect.contains([x, y])
    }

    pub fn draw(&self, ctx: &mut Context, font_path: &str, font_scale: f32) -> GameResult {
        if self.enable {
            if self.image_path.is_some() {
                self.draw_image(ctx)?;
            } else {
                self.draw_rect(ctx)?;
                self.draw_text(ctx, font_path, font_scale)?;
            }
        }
        Ok(())
    }

    /// Draw the button without text.
    fn draw_rect(&self, ctx: &mut Context) -> GameResult {
        let mesh = graphics::MeshBuilder::new()
            .rectangle(graphics::DrawMode::stroke(3.0), self.rect, self.color)?
            .build(ctx)?;
        graphics::draw(ctx, &mesh, graphics::DrawParam::default())?;
        Ok(())
    }

    /// Draw the text of the button.
    fn draw_text(&self, ctx: &mut Context, font_path: &str, font_scale: f32) -> GameResult {
        let font = graphics::Font::new(ctx, font_path)?;
        let text = graphics::Text::new((self.text, font, font_scale));
        let dest_point = match self.align {
            Align::Left => [self.rect.x, self.rect.y],
            Align::Right => [
                self.rect.x + self.rect.w - text.width(ctx),
                self.rect.y + self.rect.h - text.height(ctx),
            ],
            Align::Center => [
                self.rect.x + (self.rect.w - text.width(ctx)) / 2.0,
                self.rect.y + (self.rect.h - text.height(ctx)) / 2.0,
            ],
        };
        graphics::draw(ctx, &text, (dest_point, self.color))?;
        Ok(())
    }

    /// Draw the button without text.
    fn draw_image(&self, ctx: &mut Context) -> GameResult {
        let image = graphics::Image::new(ctx, self.image_path.unwrap()).expect("Image load error");
        let image_scale = [
            self.rect.w / image.width() as f32,
            self.rect.h / image.height() as f32,
        ];
        let dp = graphics::DrawParam::new()
            .dest(self.rect.point())
            .scale(image_scale);
        graphics::draw(ctx, &image, dp)?;
        Ok(())
    }

    /// Call the func when the button is clicked.
    pub fn clicked(&self) -> Option<&Event> {
        if self.enable {
            return Some(&self.on_click);
        }
        None
    }
    // pub fn create_next_move_button() -> Self {
    //     Button::new(
    //         "next_move",
    //         true,
    //         graphics::Rect::new(
    //             BOARD_PX_SIZE.0 + 200.0,
    //             SIDE_SCREEN_PX_SIZE.1 - 210.0,
    //             150.0,
    //             50.0,
    //         ),
    //         graphics::Color::new(0.65, 0.44, 0.78, 1.0),
    //         "->",
    //         Align::Center,
    //         Some(|gui| {
    //             gui.chess.next_move();
    //         }),
    //     )
    // }
    pub fn create_prev_move_button() -> Self {
        Button::new(
            "prev_move",
            true,
            graphics::Rect::new(
                BOARD_PX_SIZE.0 + 20.0,
                SIDE_SCREEN_PX_SIZE.1 - 210.0,
                150.0,
                50.0,
            ),
            graphics::Color::new(0.65, 0.44, 0.78, 1.0),
            "<-",
            Align::Center,
            Event::GetPrevMove,
        )
    }
}

impl fmt::Display for Button {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl fmt::Debug for Button {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}
