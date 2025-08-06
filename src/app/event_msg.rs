use crate::{
    app::{
        event_async_task_manager::TaskId,
        tea_model::{AppState, RepeatShortcutKey},
        ui_components::PopoverSelectorEvent,
    },
    sdk::{extensions::events::EventStreamHandle, OpenCodeClient, OpenCodeError},
};
use opencode_sdk::models::{Event, GetSessionByIdMessage200ResponseInner, Mode, Session};

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

    // Modes messages
    ModesLoaded(Vec<Mode>),
    ModesLoadFailed(OpenCodeError),
    CycleModeState,

    // Session messages
    SessionMessagesLoaded(Vec<GetSessionByIdMessage200ResponseInner>),
    SessionMessagesLoadFailed(OpenCodeError),
    
    // User message sending
    UserMessageSent(String), // The text that was sent
    UserMessageSendFailed(OpenCodeError),

    // Event stream messages
    EventReceived(Event),
    EventStreamConnected(EventStreamHandle),
    EventStreamDisconnected,
    EventStreamError(String),
    EventStreamReconnecting(u32), // attempt number

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
    AsyncLoadModes(OpenCodeClient),
    AsyncLoadSessionMessages(OpenCodeClient, String),
    AsyncSendUserMessage(OpenCodeClient, String, String, String, String, Option<Mode>), // client, session_id, text, provider_id, model_id, mode
    AsyncCancelTask(TaskId),
    AsyncSessionAbort,

    // Event stream commands
    AsyncStartEventStream(OpenCodeClient),
    AsyncStopEventStream,
    AsyncReconnectEventStream,

    // Batched commands for efficiency
    Batch(Vec<Cmd>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Sub {
    KeyboardInput,
    TerminalResize,
    EventStream,
}
