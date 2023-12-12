use std::{str::FromStr, thread};

use common::{board::Board, rank::Rank, square::Square};
use event::Event;
use gui::{config, styles, Gui};
use iced::{
    alignment, executor,
    widget::{
        button::StyleSheet, container, responsive, row, Button, Column, Container, Image, Row,
    },
    Alignment, Application, Command, Length, Renderer, Sandbox,
};
use logic::{movetree::pgn::STARTING_POSITION_FEN, Dispatcher};
use prelude::Result;

use crate::common::file::File;

mod common;
mod error;
mod event;
mod gui;
mod logic;
mod prelude;

struct App {}

impl Application for App {
    type Message = Event;
    type Flags = ();
    type Theme = styles::Theme;
    type Executor = executor::Default;

    fn new(flags: Self::Flags) -> (App, iced::Command<event::Event>) {
        (Self {}, Command::none())
    }

    fn title(&self) -> String {
        String::from("Ace Chess")
    }

    fn update(&mut self, message: Self::Message) -> Command<event::Event> {
        Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Renderer<styles::Theme>> {
        let board = Board::from_str(STARTING_POSITION_FEN).unwrap();
        let resp = responsive(move |size| {
            let mut board_col = Column::new().spacing(0).align_items(Alignment::Center);
            let mut board_row = Row::new().spacing(0).align_items(Alignment::Center);
            let ranks = (1..=8)
                .rev()
                .map(|r| Rank::from_str(&r.to_string()).unwrap())
                .collect::<Vec<Rank>>();
            let files = (1..=8)
                .map(|f| File::from_str(&f.to_string()).unwrap())
                .collect::<Vec<File>>();
            dbg!(&files);
            dbg!(&ranks);

            for (i, rank) in ranks.iter().enumerate() {
                for (j, file) in files.iter().enumerate() {
                    let square = Square::make_square(*file, *rank);
                    let square_content = match &board.on(square) {
                        Some((piece, color)) => {
                            let piece_color = match color {
                                common::color::Color::White => String::from("white"),
                                common::color::Color::Black => String::from("black"),
                            };
                            let piece_type = match piece {
                                common::piece::Piece::Pawn => String::from("pawn"),
                                common::piece::Piece::Knight => String::from("knight"),
                                common::piece::Piece::Bishop => String::from("bishop"),
                                common::piece::Piece::Rook => String::from("rook"),
                                common::piece::Piece::Queen => String::from("queen"),
                                common::piece::Piece::King => String::from("king"),
                            };
                            format!("{piece_color}_{piece_type}")
                        }
                        None => String::from(""),
                    };

                    let button_style = if (i + j) % 2 != 0 {
                        styles::ButtonStyle::DarkSquare
                    } else {
                        styles::ButtonStyle::LightSquare
                    };
                    board_row = board_row.push(
                        Button::new(
                            container(
                                Image::new(format!("resources/images/pieces/{square_content}.png"))
                                    .height(Length::Fill)
                                    .width(Length::Fill),
                            )
                            .align_x(alignment::Horizontal::Center)
                            .align_y(alignment::Vertical::Center),
                        )
                        .style(button_style)
                        .width((size.width / 8.) as u16)
                        .height((size.height / 8.) as u16), // .style(),
                    );
                }
                board_col = board_col.push(board_row);
                board_row = Row::new().spacing(0).align_items(Alignment::Center);
            }
            board_col.into()
        });
        Container::new(resp)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .into()
    }
}
fn main() -> iced::Result {
    let (gui_sender, logic_recv) = crossbeam_channel::bounded::<Event>(10);
    let (logic_sender, gui_recv) = crossbeam_channel::bounded::<Event>(10);

    let _ = thread::Builder::new()
        .name(String::from("logic"))
        .spawn(move || {
            let board = Board::default();
            let mut dispatcher = Dispatcher::new(board, logic_sender);
            loop {
                let event = logic_recv.recv().unwrap();
                dispatcher.dispatch(&event);
            }
        });
    App::run(iced::Settings::default())
    // let gui = Gui::new(gui_sender, gui_recv);
    // run(gui);
}

// // Run the GUI.
// pub fn run(game: Gui) {
//     let default_conf = ggez::conf::Conf {
//         window_mode: ggez::conf::WindowMode::default()
//             .dimensions(config::SCREEN_PX_SIZE.0, config::SCREEN_PX_SIZE.1),
//         window_setup: ggez::conf::WindowSetup::default()
//             .title("Chess")
//             .icon("/images/icon.png"),
//         backend: ggez::conf::Backend::default(),
//         modules: ggez::conf::ModuleConf {
//             gamepad: false,
//             audio: false,
//         },
//     };
//     let (ctx, event_loop) =
//         ggez::ContextBuilder::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_AUTHORS"))
//             .add_resource_path::<std::path::PathBuf>(
//                 [env!("CARGO_MANIFEST_DIR"), "resources"].iter().collect(),
//             )
//             .default_conf(default_conf)
//             .build()
//             .expect("Failed to build ggez context");
//
//     ggez::event::run(ctx, event_loop, game)
// }
macro_rules! rgb {
    ($r:expr, $g:expr, $b:expr) => {
        iced::Color::from_rgb($r as f32 / 255.0, $g as f32 / 255.0, $b as f32 / 255.0)
    };
}

// struct ButtonColor {
//     color: iced::Color,
// }
//
// impl StyleSheet for ButtonColor {
//     fn active(&self) -> button::Style {
//         Button::Style {
//             background: Some(iced::Background::Color(self.color)),
//             ..Default::default()
//         }
//     }
//
//     type Style;
//     // other methods in Stylesheet have a default impl
// }
