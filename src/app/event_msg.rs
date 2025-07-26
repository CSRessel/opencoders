use crate::{
    app::{
        event_async_task_manager::TaskId, tea_model::AppState, ui_components::PopoverSelectorEvent,
    },
    sdk::{OpenCodeClient, OpenCodeError},
};
use opencode_sdk::models::{GetSessionByIdMessage200ResponseInner, Session};

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
    SessionReady(Session),
    SessionInitializationFailed(OpenCodeError),
    SessionCreatedWithMessage(Session, String),
    SessionCreationFailed(OpenCodeError),

    // Session selector messages
    ShowSessionSelector,
    SessionSelectorEvent(PopoverSelectorEvent),
    SessionsLoaded(Vec<Session>),
    SessionsLoadFailed(OpenCodeError),

    // Session messages
    SessionMessagesLoaded(Vec<GetSessionByIdMessage200ResponseInner>),
    SessionMessagesLoadFailed(OpenCodeError),

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
    AsyncCreateSessionWithMessage(OpenCodeClient, String),
    AsyncLoadSessions(OpenCodeClient),
    AsyncLoadSessionMessages(OpenCodeClient, String),
    AsyncCancelTask(TaskId),

    // Batched commands for efficiency
    Batch(Vec<Cmd>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Sub {
    KeyboardInput,
}
