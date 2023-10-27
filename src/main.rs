use std::{sync::RwLock, thread};

use common::{board::Board, square::Square};
use crossbeam_channel::Receiver;
use gui::Gui;

mod common;
mod error;
mod gui;
mod prelude;

pub enum Event {
    MakeMove(Square, Square),
    CatchAll,
}

fn main() -> Result<(), anyhow::Error> {
    let board = RwLock::new(Board::default());
    let (sender, recv) = crossbeam_channel::unbounded::<Event>();

    let gui = Gui::new(board, sender);
    gui::run(gui);

    thread::spawn(logic(recv));
    Ok(())
}

fn logic(recv: Receiver<Event>) -> impl Fn() {
    move || loop {
        let command = recv
            .try_recv()
            .expect("Waiting for new commands on logic thread");
        // match command {
        //     Event::SquareClicked(x, y) => {}
        //     _ => {}
        // }
    }
}
