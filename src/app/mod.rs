mod app_program;
mod event_async_task_manager;
mod event_msg;
mod event_sync_subscriptions;
pub mod logger;
mod message_state;
mod tea_model;
mod tea_update;
mod tea_view;
mod terminal;
mod tracing_macros;
pub mod ui_components;
mod view_model_context;

pub use app_program::Program;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let program = Program::new()?;
    program.run()
}
