use nanami_protocol::{OpenClawConnectionStatus, OpenClawStatusPayload};

use crate::config::OpenClawConfig;

pub(crate) fn payload(
    config: &OpenClawConfig,
    status: OpenClawConnectionStatus,
    message: Option<&str>,
) -> OpenClawStatusPayload {
    OpenClawStatusPayload {
        status,
        gateway_url: config.gateway_url.clone(),
        message: message.map(str::to_owned),
        agent: None,
        profile: None,
    }
}
