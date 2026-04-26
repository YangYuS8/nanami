#[derive(Debug)]
pub enum OpenClawError {
    InvalidClient(reqwest::Error),
    AuthFailed,
    Disconnected,
    InvalidResponse,
    UnexpectedStatus(u16),
}

impl std::fmt::Display for OpenClawError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidClient(_) => write!(formatter, "OpenClaw client configuration failed"),
            Self::AuthFailed => write!(formatter, "OpenClaw Gateway authentication failed"),
            Self::Disconnected => write!(formatter, "OpenClaw Gateway is unreachable"),
            Self::InvalidResponse => write!(
                formatter,
                "OpenClaw Gateway returned an unsupported response"
            ),
            Self::UnexpectedStatus(status) => {
                write!(formatter, "OpenClaw Gateway returned HTTP {status}")
            }
        }
    }
}

impl std::error::Error for OpenClawError {}

pub(crate) fn error_without_secret(error: &reqwest::Error) -> String {
    if error.is_timeout() {
        "OpenClaw Gateway request timed out".into()
    } else if error.is_connect() {
        "OpenClaw Gateway is unreachable".into()
    } else {
        "OpenClaw Gateway request failed".into()
    }
}
