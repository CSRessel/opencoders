use crate::app::{
    event_msg::{Cmd, Msg},
    tea_model::{AppState, Model},
    ui_components::text_input::TextInputEvent,
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
                model.last_input = Some(submitted_text);
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

        Msg::Quit => {
            model.state = AppState::Quit;
            (model, Cmd::None)
        }
        Msg::ScrollMessageLog(direction) => {
            let new_scroll = model.message_log_scroll as i16 + direction;
            model.message_log_scroll = new_scroll.max(0) as u16;
            (model, Cmd::None)
        }
    }
}

