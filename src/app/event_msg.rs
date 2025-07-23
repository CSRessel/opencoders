use crate::{app::tea_model::AppState, sdk::{OpenCodeClient, OpenCodeError}};
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum Cmd {
    RebootTerminalWithInline(bool),
    None,
    
    // Client initialization commands
    DiscoverAndConnectClient,
    InitializeSessionForClient(OpenCodeClient),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Sub {
    KeyboardInput,
}
