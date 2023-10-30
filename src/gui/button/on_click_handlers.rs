use std::{cell::RefCell, rc::Rc};

use indextree::NodeId;

use crate::{error::Error, event::Event, gui::Gui, logic::NextMoveOptions};

use super::Button;

#[derive(Clone)]
pub struct EventHandler(pub Rc<HandlerFunction>);

pub type HandlerFunction = RefCell<dyn Fn(&mut Gui)>;

pub fn get_next_move(gui: &mut Gui) {
    let _ = gui
        .logic_channel
        .send(Event::GetNextMove(gui.displayed_node));
    match gui.receiver.recv().unwrap() {
        Event::NextMoveResponse(Ok(NextMoveOptions::Single(node))) => {
            gui.displayed_node = Some(node);
        }
        Event::NextMoveResponse(Err(Error::NoNextMove)) => {}
        Event::NextMoveResponse(Ok(NextMoveOptions::Multiple(options))) => options
            .into_iter()
            .enumerate()
            .for_each(|(idx, (node_id, notation))| {
                gui.buttons.push(Button::create_next_move_option_button(
                    notation,
                    idx,
                    Rc::new(RefCell::new(move |gui: &mut Gui| go_to_node(gui, node_id))),
                ))
            }),
        _ => get_next_move(gui),
    };
}

pub fn go_to_node(gui: &mut Gui, node_id: NodeId) {
    let _ = gui.logic_channel.send(Event::GoToNode(node_id));
    match gui.receiver.recv().unwrap() {
        Event::NewDisplayNode(Ok(node)) => {
            gui.displayed_node = Some(node);
            gui.init_buttons();
        }
        _ => go_to_node(gui, node_id),
    }
}
pub fn get_prev_move(gui: &mut Gui) {
    if let Some(node) = gui.displayed_node {
        let _ = gui.logic_channel.send(Event::GetPrevMove(node));
        match gui.receiver.recv().unwrap() {
            Event::NewDisplayNode(Ok(node)) => {
                gui.displayed_node = Some(node);
            }
            Event::NewDisplayNode(Err(Error::NoPrevMove)) => {
                gui.displayed_node = None;
            }
            _ => get_prev_move(gui),
        };
    }
}
