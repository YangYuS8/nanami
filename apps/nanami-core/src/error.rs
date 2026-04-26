use nanami_protocol::{ErrorPayload, ErrorSeverity};

pub(crate) fn chat_error(code: &str, message: &str, action_hint: Option<&str>) -> ErrorPayload {
    ErrorPayload {
        task_id: None,
        severity: ErrorSeverity::Error,
        code: code.into(),
        message: message.into(),
        action_hint: action_hint.map(str::to_owned),
    }
}
