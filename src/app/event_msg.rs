use crate::{
    app::{
        event_async_task_manager::TaskId,
        tea_model::{AppState, RepeatShortcutKey},
        ui_components::PopoverSelectorEvent,
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
    ValidateScrollPosition(u16, u16), // viewport_height, viewport_width

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

    // TODO
    // Session interactions
    SessionAbort,

    // Task lifecycle messages
    TaskStarted(TaskId, String),
    TaskCompleted(TaskId),
    TaskFailed(TaskId, String),

    // Progress reporting messages
    ConnectionProgress(f32),
    SessionProgress(f32),

    // View state management
    MarkMessagesViewed,

    // Terminal events
    TerminalResize(u16, u16), // width, height
    ChangeInlineHeight(u16),  // new height for inline mode

    // Unified repeat shortcut timeout events
    RepeatShortcutPressed(RepeatShortcutKey),
    ClearTimeout,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Cmd {
    None,

    // Terminal or crossterm related side-effects
    TerminalAutoResize,             // trigger autoresize for any viewport changes
    TerminalRebootWithInline(bool), // reinitialize for new viewport
    TerminalResizeInlineViewport(u16), // new height for inline mode
    TerminalScrollPastHeight,       // scroll past any manual stdio output

    // Async commands that don't block
    AsyncSpawnClientDiscovery,
    AsyncSpawnSessionInit(OpenCodeClient),
    AsyncCreateSessionWithMessage(OpenCodeClient, String),
    AsyncLoadSessions(OpenCodeClient),
    AsyncLoadSessionMessages(OpenCodeClient, String),
    AsyncCancelTask(TaskId),
    AsyncSessionAbort,

    // Batched commands for efficiency
    Batch(Vec<Cmd>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Sub {
    KeyboardInput,
    TerminalResize,
}
