mod components;
mod model;
mod msg;
mod program;
mod subscriptions;
mod terminal;
mod update;
mod view;

pub use program::Program;

const INLINE_MODE: bool = false;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let program = Program::new(INLINE_MODE)?;
    program.run()
}
