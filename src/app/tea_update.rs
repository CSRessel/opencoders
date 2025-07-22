use crate::app::{
    event_msg::{Cmd, Msg},
    tea_model::{AppState, Model},
    ui_components::text_input::TextInputEvent,
};
use opencode_sdk::models::{
    GetSessionByIdMessage200ResponseInner, Message, Part, TextPart, UserMessage,
};

pub fn update(mut model: Model, msg: Msg) -> (Model, Cmd) {
    match msg {
        Msg::KeyPressed(c) => {
            if let Some(submitted_text) = model.text_input.handle_event(TextInputEvent::Insert(c)) {
                model.last_input = Some(submitted_text);
            }
            (model, Cmd::None)
        }

        Msg::Backspace => {
            model.text_input.handle_event(TextInputEvent::Delete);
            (model, Cmd::None)
        }

        Msg::SubmitInput => {
            if let Some(submitted_text) = model.text_input.handle_event(TextInputEvent::Submit) {
                model.input_history.push(submitted_text.clone());
                model.last_input = Some(submitted_text.clone());

                model
                    .message_log
                    .create_and_push_user_message(&submitted_text)
            }
            (model, Cmd::None)
        }

        Msg::ClearInput => {
            model.text_input.clear();
            model.last_input = None;
            model.input_history.clear();
            model.printed_to_stdout_count = 0;
            (model, Cmd::None)
        }

        Msg::ChangeState(new_state) => {
            model.state = new_state;
            if matches!(model.state, AppState::Welcome) {
                model.text_input.clear();
                model.last_input = None;
                model.input_history.clear();
                model.printed_to_stdout_count = 0;
            }
            (model, Cmd::None)
        }

        Msg::ChangeInline => {
            let new_inline = !model.init.inline_mode().clone();
            (model, Cmd::RebootTerminalWithInline(new_inline))
        }

        Msg::Quit => {
            model.state = AppState::Quit;
            (model, Cmd::None)
        }
        Msg::ScrollMessageLog(direction) => {
            model.message_log.move_message_log_scroll(&direction);
            (model, Cmd::None)
        }
    }
}
