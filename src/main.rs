use crate::common::file::File;
use common::{board::Board, rank::Rank, square::Square};
use iced::{
    alignment,
    event::Event,
    executor,
    keyboard::{self, Modifiers},
    widget::{container, responsive, row, Button, Column, Container, Image, Row},
    Alignment, Application, Command, Length, Renderer, Subscription,
};

use logic::movetree::{MoveTree, NextMoveOptions};
use message::Message;
use prelude::Result;
use std::str::FromStr;

mod common;
mod error;
mod logic;
mod message;
mod prelude;
mod styles;

#[derive(Default)]
struct App {
    board: Board,
    selected_square: Option<Square>,
    move_tree: MoveTree,
    displayed_node: Option<indextree::NodeId>,
}

fn main() -> iced::Result {
    App::run(iced::Settings::default())
}

impl Application for App {
    type Message = Message;
    type Flags = ();
    type Theme = styles::Theme;
    type Executor = executor::Default;

    fn new(_flags: Self::Flags) -> (App, iced::Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Ace Chess")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::SelectSquare(s) => self.selected_square = Some(s),
            Message::MakeMove(from, to, displayed_node) => {
                let m = common::r#move::Move::new(from, to);
                if self.board.is_legal(m) {
                    let new_node = self.move_tree.add_new_move(m, displayed_node, &self.board);
                    self.board = self.board.update(m);
                    self.selected_square = None;
                    self.displayed_node = Some(new_node);
                } else if self.board.color_on_is(to, self.board.side_to_move()) {
                    self.selected_square = Some(to);
                } else {
                    self.selected_square = None;
                }
            }
            Message::Event(e) => match e {
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key_code: keyboard::KeyCode::Left,
                    ..
                }) => {
                    if let Some(n) = self.displayed_node {
                        match self.move_tree.get_prev_move(n) {
                            Ok((id, fen)) => {
                                self.board = Board::from_str(fen)
                                    .expect("Failed to load board from prev_move fen");
                                self.displayed_node = Some(id);
                            }
                            Err(e) => {
                                eprintln!("Could not get prev move: {:?}", e);
                            }
                        }
                    }
                }
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key_code: keyboard::KeyCode::Right,
                    ..
                }) => match self.move_tree.get_next_move(self.displayed_node) {
                    Ok(NextMoveOptions::Single(id, fen)) => {
                        self.board =
                            Board::from_str(&fen).expect("Failed to load board from next_move fen");
                        self.displayed_node = Some(id);
                    }
                    Ok(NextMoveOptions::Multiple(options)) => {
                        let (id, _fen) = options.first().unwrap();
                        self.board =
                            Board::from_str(self.move_tree.get_tree()[*id].get().fen.as_str())
                                .expect("Failed to load board from node fen");
                        self.displayed_node = Some(*id);
                    }
                    Err(_) => eprintln!("Could not get next move"),
                },
                _ => {}
            },
            _ => {}
        }
        Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Renderer<styles::Theme>> {
        let resp = responsive(move |size| {
            let board_width = size.width * 0.75;
            let mut board_col = Column::new().spacing(0).align_items(Alignment::Center);
            let mut board_row = Row::new().spacing(0).align_items(Alignment::Center);
            let ranks = (1..=8)
                .rev()
                .map(|r| Rank::from_str(&r.to_string()).unwrap())
                .collect::<Vec<Rank>>();
            let files = (1..=8)
                .map(|f| File::from_str(&f.to_string()).unwrap())
                .collect::<Vec<File>>();

            for (i, rank) in ranks.iter().enumerate() {
                for (j, file) in files.iter().enumerate() {
                    let square = Square::make_square(*file, *rank);
                    let square_content = match &self.board.on(square) {
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
                        if self.selected_square == Some(square) {
                            styles::ButtonStyle::SelectedDarkSquare
                        } else {
                            styles::ButtonStyle::DarkSquare
                        }
                    } else if self.selected_square == Some(square) {
                        styles::ButtonStyle::SelectedLightSquare
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
                        .on_press(if let Some(s) = self.selected_square {
                            message::Message::MakeMove(s, square, self.displayed_node)
                        } else {
                            message::Message::SelectSquare(square)
                        })
                        .style(button_style)
                        .width((board_width / 8.) as u16)
                        .height((size.height / 8.) as u16), // .style(),
                    );
                }
                board_col = board_col.push(board_row);
                board_row = Row::new().spacing(0).align_items(Alignment::Center);
            }
            let controls = row!(
                Button::new(
                    Container::new("<-")
                        .align_x(alignment::Horizontal::Center)
                        .align_y(alignment::Vertical::Center),
                )
                .on_press(Message::Event(Event::Keyboard(
                    keyboard::Event::KeyPressed {
                        key_code: keyboard::KeyCode::Left,
                        modifiers: Modifiers::SHIFT
                    }
                )))
                .style(styles::ButtonStyle::Normal)
                // .height(Length::Fill)
                .width(Length::Fill),
                Button::new(
                    Container::new("->")
                        .align_x(alignment::Horizontal::Center)
                        .align_y(alignment::Vertical::Center)
                ) // .height(Length::Fill)
                .on_press(Message::Event(Event::Keyboard(
                    keyboard::Event::KeyPressed {
                        key_code: keyboard::KeyCode::Right,
                        modifiers: Modifiers::SHIFT
                    }
                )))
                .style(styles::ButtonStyle::Normal)
                .width(Length::Fill),
            )
            .width(size.width * 0.3)
            // .spacing(5)
            .align_items(Alignment::End);

            row!(board_col, controls).into()
        });
        Container::new(resp)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
    fn subscription(&self) -> Subscription<Message> {
        iced::subscription::events().map(Message::Event)
    }
}
