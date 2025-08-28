use crate::{
    app::{
        event_async_task_manager::TaskId,
        tea_model::{AppModalState, RepeatShortcutKey},
        ui_components::{MsgTextArea, MsgModalSessionSelector, MsgModalFileSelector},
    },
    sdk::{extensions::events::EventStreamHandle, OpenCodeClient, OpenCodeError},
};
use opencode_sdk::models::{ConfigAgent, Event, Model, Session, SessionMessages200ResponseInner};

#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    ChangeState(AppModalState),
    Quit,
    ScrollMessageLog(i16),
    ScrollMessageLogHorizontal(i16),
    ValidateScrollPosition(u16, u16), // viewport_height, viewport_width
    SubmitTextInput,

    // Client initialization messages
    InitializeClient,
    ClientConnected(OpenCodeClient),
    ClientConnectionFailed(OpenCodeError),

    // Session management messages
    SessionReady(Session),
    SessionInitializationFailed(OpenCodeError),
    SessionCreatedWithMessage(Session, String),
    SessionCreationFailed(OpenCodeError),
    SessionAbort,

    // Leader actions that reset interval
    LeaderChangeInline,
    LeaderShowSessionSelector,

    // Session selector messages
    SessionsLoaded(Vec<Session>),
    SessionsLoadFailed(OpenCodeError),

    // Modes messages
    ModesLoaded(ConfigAgent),
    ModesLoadFailed(OpenCodeError),
    CycleModeState,

    // Session messages
    SessionMessagesLoaded(Vec<SessionMessages200ResponseInner>),
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

    // Task lifecycle messages
    TaskStarted(TaskId, String),
    TaskCompleted(TaskId),
    TaskFailed(TaskId, String),
    RecordActiveTaskCount(usize),

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

    // Verbosity control
    ToggleVerbosity,

    // File status loading
    FileStatusLoaded(Vec<opencode_sdk::models::File>),
    FileStatusLoadFailed(OpenCodeError),

    // Component messages
    TextArea(MsgTextArea),
    ModalSessionSelector(MsgModalSessionSelector),
    ModalFileSelector(MsgModalFileSelector),
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
    AsyncLoadFileStatus(OpenCodeClient),
    AsyncSendUserMessage(
        OpenCodeClient,
        String,
        String,
        String,
        String,
        String,
        Option<String>,
    ), // client, session_id, message_id, text, provider_id, model_id, mode
    AsyncCancelTask(TaskId),
    AsyncSessionAbort,

    // Event stream commands
    AsyncStartEventStream(OpenCodeClient),
    AsyncStopEventStream,
    AsyncReconnectEventStream,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CmdOrBatch<T> {
    Single(T),
    Batch(Vec<T>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Sub {
    KeyboardInput,
    TerminalResize,
    EventStream,
}
