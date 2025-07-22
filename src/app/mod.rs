mod app_program;
mod event_msg;
mod event_subscriptions;
mod tea_model;
mod tea_update;
mod tea_view;
mod ui_components;

pub use app_program::Program;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let program = Program::new()?;
    program.run()
}
