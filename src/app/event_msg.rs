use crate::{
    app::{event_async_task_manager::TaskId, tea_model::AppState},
    sdk::{OpenCodeClient, OpenCodeError},
};
use opencode_sdk::models::Session;

#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    KeyPressed(char),
    Backspace,
    SubmitInput,
    ClearInput,
    ChangeInline,
    ChangeState(AppState),
    Quit,
    ScrollMessageLog(i16),
    ScrollMessageLogHorizontal(i16),

    // Client initialization messages
    InitializeClient,
    ClientConnected(OpenCodeClient),
    ClientConnectionFailed(OpenCodeError),

    // Session management messages
    InitializeSession,
    SessionReady(Session),
    SessionInitializationFailed(OpenCodeError),

    // Task lifecycle messages
    TaskStarted(TaskId, String),
    TaskCompleted(TaskId),
    TaskFailed(TaskId, String),

    // Progress reporting messages
    ConnectionProgress(f32),
    SessionProgress(f32),

    // View state management
    MarkMessagesViewed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Cmd {
    RebootTerminalWithInline(bool),
    None,

    // Async commands that don't block
    AsyncSpawnClientDiscovery,
    AsyncSpawnSessionInit(OpenCodeClient),
    AsyncCancelTask(TaskId),

    // Batched commands for efficiency
    Batch(Vec<Cmd>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Sub {
    KeyboardInput,
}
