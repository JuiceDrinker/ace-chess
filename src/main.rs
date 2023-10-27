use std::{
    sync::{Arc, RwLock},
    thread,
};

use common::{board::Board, square::Square};
use event::Event;
use gui::{config, Gui};

mod common;
mod error;
mod event;
mod gui;
mod logic;
mod prelude;

fn main() -> Result<(), anyhow::Error> {
    let board = Arc::new(RwLock::new(Board::default()));
    let (sender, recv) = crossbeam_channel::unbounded::<Event>();

    let gui = Gui::new(board.clone(), sender);
    thread::spawn(move || loop {
        let event = recv
            .recv()
            .expect("Waiting for new commands on logic thread");
        dbg!("Got event", &event);
        logic::dispatcher(event, board.clone());
    });
    run(gui);
    Ok(())
}

// Run the GUI.
pub fn run(game: Gui) {
    let default_conf = ggez::conf::Conf {
        window_mode: ggez::conf::WindowMode::default()
            .dimensions(config::SCREEN_PX_SIZE.0, config::SCREEN_PX_SIZE.1),
        window_setup: ggez::conf::WindowSetup::default()
            .title("Chess")
            .icon("/images/icon.png"),
        backend: ggez::conf::Backend::default(),
        modules: ggez::conf::ModuleConf {
            gamepad: false,
            audio: false,
        },
    };
    let (ctx, event_loop) =
        ggez::ContextBuilder::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_AUTHORS"))
            .add_resource_path::<std::path::PathBuf>(
                [env!("CARGO_MANIFEST_DIR"), "resources"].iter().collect(),
            )
            .default_conf(default_conf)
            .build()
            .expect("Failed to build ggez context");

    ggez::event::run(ctx, event_loop, game)
}
