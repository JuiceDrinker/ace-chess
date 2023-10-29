mod button;
pub mod config;
mod theme;

use ggez::event::{KeyCode, KeyMods, MouseButton};
use ggez::{event, graphics, Context, GameError};
use indextree::NodeId;

use self::button::Button;
use self::config::BOARD_CELL_PX_SIZE;
use self::theme::Theme;
use crate::common::board::Board;
use crate::common::square::{Square, ALL_SQUARES};
use crate::gui::config::BOARD_PX_SIZE;
use crate::prelude::BOARD_SIZE;
use crate::Event;

type GameResult<T = ()> = Result<T, GameError>;
pub struct Gui {
    selected_square: Option<Square>,
    theme: Theme,
    buttons: Vec<Button>,
    displayed_node: Option<NodeId>,
    logic_channel: crossbeam_channel::Sender<Event>,
    receiver: crossbeam_channel::Receiver<Event>,
}

impl Gui {
    pub fn new(
        logic_channel: crossbeam_channel::Sender<Event>,
        gui_channel: crossbeam_channel::Receiver<Event>,
    ) -> Self {
        let mut gui = Self {
            buttons: vec![],
            displayed_node: None,
            selected_square: None,
            logic_channel,
            receiver: gui_channel,
            theme: Theme::default(),
        };
        gui.buttons
            .push(Button::create_prev_move_button(get_prev_move));
        gui
    }

    pub fn board(&self) -> Option<Board> {
        let _ = self.logic_channel.send(Event::RequestBoard);
        match self.receiver.recv().unwrap() {
            Event::SendBoard(board) => Some(board),
            _ => None,
        }
    }

    // Draw all of the board side.
    fn draw_board(&self, ctx: &mut Context) -> GameResult {
        self.draw_empty_board(ctx)?;
        self.draw_legal_moves(ctx)?;
        self.draw_content_board(ctx)?;
        Ok(())
    }

    /// Draw the empty chess board (without pieces).
    fn draw_empty_board(&self, ctx: &mut Context) -> GameResult {
        for y in 0..BOARD_SIZE.1 {
            for x in 0..BOARD_SIZE.0 {
                let color_index = if (x % 2 == 1 && y % 2 == 1) || (x % 2 == 0 && y % 2 == 0) {
                    0
                } else {
                    1
                };
                let mesh = graphics::MeshBuilder::new()
                    .rectangle(
                        graphics::DrawMode::fill(),
                        graphics::Rect::new(
                            x as f32 * BOARD_CELL_PX_SIZE.0,
                            y as f32 * BOARD_CELL_PX_SIZE.1,
                            BOARD_CELL_PX_SIZE.0,
                            BOARD_CELL_PX_SIZE.1,
                        ),
                        self.theme.board_color[color_index],
                    )?
                    .build(ctx)?;
                graphics::draw(ctx, &mesh, graphics::DrawParam::default())?;
            }
        }
        Ok(())
    }

    fn draw_legal_moves(&self, ctx: &mut Context) -> GameResult {
        if self.theme.valid_moves_color.is_some() {
            if let Some(square) = self.selected_square {
                let _ = self.logic_channel.send(Event::GetLegalMoves(square));
                match self.receiver.recv().unwrap() {
                    Event::SendLegalMoves(dest) => {
                        for d in dest {
                            let (x, y) = d.as_screen_coords();
                            let mesh = graphics::MeshBuilder::new()
                                .rectangle(
                                    graphics::DrawMode::fill(),
                                    graphics::Rect::new(
                                        x,
                                        y,
                                        BOARD_CELL_PX_SIZE.0,
                                        BOARD_CELL_PX_SIZE.1,
                                    ),
                                    self.theme.valid_moves_color.unwrap(),
                                )?
                                .build(ctx)?;
                            graphics::draw(ctx, &mesh, graphics::DrawParam::default())?;
                        }
                    }
                    _ => {
                        let _ = self.draw_legal_moves(ctx);
                    }
                }
            }
        }
        Ok(())
    }

    /// Draw pieces on the board.
    fn draw_content_board(&self, ctx: &mut Context) -> GameResult {
        let mut path;
        let mut image;
        for square in ALL_SQUARES {
            if let Some(board) = self.board() {
                if let Some((piece, color)) = board.on(square) {
                    path = self.theme.piece_path[color.as_index()][piece.as_index()];
                    image = graphics::Image::new(ctx, path).expect("Image load error");
                    let (x, y) = square.as_screen_coords();
                    let dest_point = [x, y];
                    let image_scale = [0.5, 0.5];
                    let dp = graphics::DrawParam::new()
                        .dest(dest_point)
                        .scale(image_scale);
                    graphics::draw(ctx, &image, dp)?;
                }
            } else {
                return self.draw_content_board(ctx);
            }
        }
        Ok(())
    }
    /// Base function to call when a user click on the screen.
    pub fn click(&mut self, x: f32, y: f32) {
        if x < BOARD_PX_SIZE.0 {
            self.click_on_board(x, y);
        } else {
            for button in self.buttons.clone().iter() {
                if button.contains(x, y) {
                    button.clicked(self);
                }
            }
        }
    }

    // React when the user click on the board screen.
    //
    // It is the callers responsibility to ensure the coordinate is in the board.
    fn click_on_board(&mut self, x: f32, y: f32) {
        match self.selected_square {
            Some(s) => {
                let _ = self.logic_channel.send(Event::MakeMove(
                    s,
                    Square::from_screen(x, y),
                    self.displayed_node,
                ));
                let new_node = match self.receiver.recv().unwrap() {
                    Event::NewNodeAppended(node) => node,
                    _ => None,
                };
                self.selected_square = None;
                if new_node.is_some() {
                    self.displayed_node = new_node;
                };
            }
            None => {
                self.selected_square = Some(Square::from_screen(x, y));
            }
        };
        // self.logic_channel.send(Event::SquareClicked(x, y));
        // let current_square = Square::from_screen(x, y);
        // match self.square_focused {
        //     Some(square_selected) => self.chess.play(square_selected, current_square),
        //     None => {
        //         if self
        //             .chess
        //             .board
        //             .color_on_is(current_square, self.chess.board.side_to_move())
        //         {
        //             self.chess.square_focused = Some(current_square);
        //         }
        //     }
        // }
    }

    fn draw_side(&self, ctx: &mut Context) -> GameResult {
        for button in self.buttons.iter() {
            button.draw(ctx, self.theme.font_path, self.theme.font_scale)?;
        }
        Ok(())
    }
}

impl event::EventHandler<GameError> for Gui {
    /// Update will happen on every frame before it is drawn.
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // if self.chess.state.is_finish() {
        //     for button in self.buttons.iter_mut() {
        //         match button.id {
        //             "reset" | "theme" => {}
        //             _ => button.disable(),
        //         }
        //     }
        // }
        Ok(())
    }

    /// Render the game's current state.
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // First we clear the screen and set the background color
        graphics::clear(ctx, self.theme.background_color);

        // Draw the board and the side screen (that contains all button/info)
        self.draw_board(ctx)?;
        self.draw_side(ctx)?;

        // Finally we call graphics::present to cycle the gpu's framebuffer and display
        // the new frame we just drew.
        graphics::present(ctx)?;

        // And return success.
        Ok(())
    }

    /// Called every time a mouse button gets pressed
    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if button == MouseButton::Left {
            self.click(x, y);
        }
    }

    // Change the [`ggez::input::mouse::CursorIcon`] when the mouse is on a button.
    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        if x > BOARD_PX_SIZE.0 {
            let mut on_button = false;
            for button in self.buttons.iter() {
                if button.contains(x, y) {
                    on_button = true;
                    break;
                }
            }
            if on_button {
                ggez::input::mouse::set_cursor_type(ctx, ggez::input::mouse::CursorIcon::Hand);
            } else {
                ggez::input::mouse::set_cursor_type(ctx, ggez::input::mouse::CursorIcon::Default);
            }
        }
    }

    /// Called every time a key gets pressed.
    ///
    /// # Keys
    ///
    /// |  Keys  |          Actions           |
    /// |--------|----------------------------|
    /// | Escape | Quit the game              |
    /// | R      | Reset the game and buttons |
    /// | CTRL+Z | Undo                       |
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            KeyCode::Escape => event::quit(ctx),
            // KeyCode::NavigateForward => self.chess.next_move(),
            // KeyCode::R => self.reset(),
            KeyCode::NavigateBackward => get_prev_move(self),
            _ => {}
        };
    }
}
fn get_prev_move(gui: &mut Gui) {
    println!("Am I here too?");
    if let Some(node) = gui.displayed_node {
        let _ = gui.logic_channel.send(Event::GetPrevMove(node));
        match gui.receiver.recv().unwrap() {
            Event::NewDisplayNode(node) => {
                gui.displayed_node = node;
            }
            _ => get_prev_move(gui),
        };
    }
}
