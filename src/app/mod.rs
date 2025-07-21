mod app_program;
mod event_msg;
mod event_subscriptions;
mod tea_model;
mod tea_update;
mod tea_view;
mod ui_components;
mod ui_terminal;

pub use app_program::Program;

const INLINE_MODE: bool = false;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let program = Program::new(INLINE_MODE)?;
    program.run()
}
