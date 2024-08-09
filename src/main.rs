use crate::{
    common::file::File,
    logic::movetree::pgn::{lexer::tokenize, parser::PgnParser},
};
use common::{board::Board, r#move::Move, rank::Rank, square::Square};
use iced::{
    alignment, clipboard, executor, keyboard,
    widget::{self, container, responsive, row, Button, Column, Container, Image, Row, Text},
    Alignment, Application, Command, Element, Length, Subscription,
};

use logic::movetree::{MoveTree, NextMoveOptions};
use message::Message;
use prelude::Result;
use std::str::FromStr;
use views::modal::Modal;

mod common;
mod error;
mod logic;
mod message;
mod prelude;
mod styles;
mod views;

struct App {
    board: Board,
    selected_square: Option<Square>,
    move_tree: MoveTree,
    displayed_node: indextree::NodeId,
    next_move_options: Option<Vec<(indextree::NodeId, String)>>,
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
        let move_tree = MoveTree::new();
        let displayed_node = move_tree.game_start();
        let app = Self {
            board: Board::default(),
            selected_square: None,
            move_tree,
            displayed_node,
            next_move_options: None,
        };
        (app, Command::none())
    }

    fn title(&self) -> String {
        String::from("Ace Chess")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::SelectSquare(square) => self.selected_square = Some(square),
            Message::MakeMove(attempted_move, displayed_node) => {
                dbg!(attempted_move);
                dbg!("getting here 3?");
                if self.board.is_legal(attempted_move) {
                    dbg!("getting here 2?");
                    if let Ok(cmove) = attempted_move.try_into_cmove(self.board) {
                        dbg!("getting here ");
                        self.board = self.board.update(attempted_move);
                        let new_node = self.move_tree.add_new_move(
                            cmove,
                            displayed_node,
                            self.board.to_string(),
                        );
                        self.selected_square = None;
                        self.displayed_node = new_node;
                    } else {
                        self.selected_square = None;
                    }
                    // If Illegal move then check whether own piece on square
                } else if self
                    .board
                    .color_on_is(attempted_move.to, self.board.side_to_move())
                {
                    // If own piece on square, prime this square to move
                    self.selected_square = Some(attempted_move.to);
                } else {
                    self.selected_square = None;
                }
            }
            Message::GoPrevMove => {
                dbg!("am I here or what");
                let (id, fen) = self.move_tree.get_prev_move(self.displayed_node);
                self.board =
                    Board::from_str(&fen).expect("Failed to load board from prev_move fen");
                self.displayed_node = id;
                dbg!("am I here or what");
            }
            Message::GoNextMove => {
                dbg!(&self.move_tree);
                match NextMoveOptions::new(self.move_tree.get_next_move(self.displayed_node)) {
                    Ok(NextMoveOptions::Single(id, fen)) => {
                        self.board =
                            Board::from_str(&fen).expect("Failed to load board from next_move fen");
                        self.displayed_node = id;
                    }
                    Ok(NextMoveOptions::Multiple(options)) => {
                        self.next_move_options = Some(options);
                        return widget::focus_next();
                    }
                    Err(_) => eprintln!("Could not get next move"),
                }
            }
            Message::GoToNode(id) => {
                if let Some(fen) = self.move_tree.get_fen_for_node(id) {
                    self.board =
                        Board::from_str(fen).expect("Failed to load board from next_move fen");
                };
                self.next_move_options = None;
                self.displayed_node = id;
            }
            Message::InitLoadPgn => {
                return clipboard::read(|content| {
                    if let Some(content) = content {
                        Message::LoadPgn(content)
                    } else {
                        Message::LoadPgn("This will fail".to_string())
                    }
                })
            }
            Message::LoadPgn(pgn) => {
                dbg!("Im here I swear");
                let tokens = tokenize(&pgn);
                if let Ok(parsed) = PgnParser::new(tokens.iter()).parse() {
                    dbg!("Im here I swear 2 ");
                    self.move_tree = parsed.clone();
                    dbg!(parsed.clone());
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message, styles::Theme> {
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
                        .on_press(if let Some(selected_square) = self.selected_square {
                            message::Message::MakeMove(
                                Move {
                                    to: square,
                                    from: selected_square,
                                },
                                self.displayed_node,
                            )
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

            // let move_text = row!(Text::new(self.move_tree.generate_pgn()))
            //     .width(size.width * 0.3)
            //     // .spacing(5)
            //     .align_items(Alignment::End);

            // let content = row!(board_col, move_text);
            let content = row!(board_col);

            if let Some(next_opts) = &self.next_move_options {
                let mut row = Row::new().spacing(2).align_items(Alignment::Center);
                row = row.extend(next_opts.iter().map(|(node, notation)| {
                    Button::new(
                        Container::new(Text::new(notation))
                            .align_x(alignment::Horizontal::Center)
                            .align_y(alignment::Vertical::Center),
                    )
                    .on_press(Message::GoToNode(*node))
                    .style(styles::ButtonStyle::Normal)
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .into()
                }));
                let modal = container(row).width(300).padding(10);
                return Modal::new(content, modal).into();
            } else {
                content.into()
            }
        });
        return Container::new(resp)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }

    fn subscription(&self) -> Subscription<Message> {
        keyboard::on_key_press(|key, modifiers| match (key.as_ref(), modifiers) {
            (keyboard::Key::Named(keyboard::key::Named::ArrowLeft), _) => Some(Message::GoPrevMove),
            (keyboard::Key::Named(keyboard::key::Named::ArrowRight), _) => {
                Some(Message::GoNextMove)
            }
            (keyboard::Key::Character("v"), modifier) if modifier.command() => {
                dbg!("TRIGgefr");
                Some(Message::InitLoadPgn)
            }
            _ => None,
        })
    }
}
