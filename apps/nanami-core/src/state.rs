use axum::http::StatusCode;
use nanami_permission::PermissionManager;
use nanami_protocol::{ErrorPayload, EventEnvelope, ProjectMetadata};
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use crate::openclaw::OpenClawService;

pub(crate) const DEFAULT_OPENCLAW_TIMEOUT_MS: u64 = 3000;
pub(crate) const MANIFEST_PREVIEW_MAX_BYTES: u64 = 8 * 1024;

pub(crate) type NanamiEventStream =
    Pin<Box<dyn tokio_stream::Stream<Item = Result<EventEnvelope, ErrorPayload>> + Send>>;
pub(crate) type JsonErrorResponse = (StatusCode, [(&'static str, &'static str); 1], String);

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) openclaw: Arc<dyn OpenClawService>,
    pub(crate) permission_manager: Arc<Mutex<PermissionManager>>,
    pub(crate) selected_project: Arc<Mutex<Option<ProjectMetadata>>>,
}
