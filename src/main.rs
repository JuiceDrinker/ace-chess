use gui::Gui;

mod gui;
mod prelude;

fn main() -> Result<(), anyhow::Error> {
    let gui = Gui::new();
    gui::run(gui);
    Ok(())
}
