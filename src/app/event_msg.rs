use crate::{
    app::{
        event_async_task_manager::TaskId,
        tea_model::{AppModalState, RepeatShortcutKey},
        ui_components::{MsgModalFileSelector, MsgModalSessionSelector, MsgTextArea},
    },
    sdk::{extensions::events::EventStreamHandle, OpenCodeClient, OpenCodeError},
};
use opencode_sdk::models::{ConfigAgent, Event, Model, Session, SessionMessages200ResponseInner};

type OpenCodeResponse<T> = Result<T, OpenCodeError>;

#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    // State changes
    InitializeClient,
    SessionAbort,
    ChangeState(AppModalState),
    Quit,

    // Major input events
    ScrollMessageLog(i16),
    ScrollMessageLogHorizontal(i16),
    ValidateScrollPosition(u16, u16), // viewport_height, viewport_width
    SubmitTextInput,
    CycleModeState,
    ToggleVerbosity,
    LeaderChangeInline,
    LeaderShowSessionSelector,
    MarkMessagesViewed,

    // Unified repeat shortcut timeout events
    RepeatShortcutPressed(RepeatShortcutKey),
    ClearTimeout,

    // Client initialization messages
    ResponseClientConnect(OpenCodeResponse<OpenCodeClient>),
    ResponseSessionInit(OpenCodeResponse<Session>),
    ResponseSessionCreateWithMessage(OpenCodeResponse<(Session, String)>),
    ResponseSessionsLoad(OpenCodeResponse<Vec<Session>>),
    ResponseModesLoad(OpenCodeResponse<ConfigAgent>),
    ResponseSessionMessagesLoad(OpenCodeResponse<Vec<SessionMessages200ResponseInner>>),
    ResponseUserMessageSend(OpenCodeResponse<String>),
    ResponseFileStatusesLoad(OpenCodeResponse<Vec<opencode_sdk::models::File>>),

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

    // Terminal events
    TerminalResize(u16, u16), // width, height
    ChangeInlineHeight(u16),  // new height for inline mode

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
