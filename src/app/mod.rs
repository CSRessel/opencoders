#![allow(unused)]

mod app_program;
pub mod error;
pub mod event_async_task_manager;
pub mod event_msg;
pub mod event_sync_subscriptions;
pub mod logger;
pub mod message_state;
pub mod tea_model;
pub mod tea_update;
pub mod tea_view;
pub mod terminal;
pub mod text_wrapper;
pub mod ui_components;
pub mod view_model_context;

pub use app_program::Program;
pub use error::Result;

pub fn run() -> Result<()> {
    let program = Program::new()?;
    program.run()
}
