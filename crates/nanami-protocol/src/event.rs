use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    ChatResponse, ErrorPayload, OpenClawStatusPayload, PermissionRequestPayload,
    PermissionResolvedPayload, PersonaStatePayload, SandboxArtifactPayload,
    SandboxCompletedPayload, SandboxOutputPayload, SandboxStartedPayload, SandboxUpdatedPayload,
    SessionStatus, TaskCompletedPayload, TaskStartedPayload, TaskUpdatedPayload,
    ToolCompletedPayload, ToolOutputPayload, ToolStartedPayload, WorkflowCompletedPayload,
    WorkflowPatchProposedPayload, WorkflowStartedPayload, WorkflowStepPayload,
    WorkflowTestResultPayload,
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum Event {
    #[serde(rename = "session.started")]
    SessionStarted { session_id: String, title: String },
    #[serde(rename = "session.updated")]
    SessionUpdated {
        session_id: String,
        status: SessionStatus,
    },
    #[serde(rename = "openclaw.status")]
    OpenClawStatus(OpenClawStatusPayload),
    #[serde(rename = "message.user")]
    MessageUser {
        session_id: String,
        message_id: String,
        content: String,
    },
    #[serde(rename = "message.delta")]
    MessageDelta {
        session_id: String,
        message_id: String,
        delta: String,
    },
    #[serde(rename = "message.completed")]
    MessageCompleted(ChatResponse),
    #[serde(rename = "task.started")]
    TaskStarted(TaskStartedPayload),
    #[serde(rename = "task.updated")]
    TaskUpdated(TaskUpdatedPayload),
    #[serde(rename = "task.completed")]
    TaskCompleted(TaskCompletedPayload),
    #[serde(rename = "tool.started")]
    ToolStarted(ToolStartedPayload),
    #[serde(rename = "tool.output")]
    ToolOutput(ToolOutputPayload),
    #[serde(rename = "tool.completed")]
    ToolCompleted(ToolCompletedPayload),
    #[serde(rename = "sandbox.started")]
    SandboxStarted(SandboxStartedPayload),
    #[serde(rename = "sandbox.updated")]
    SandboxUpdated(SandboxUpdatedPayload),
    #[serde(rename = "sandbox.output")]
    SandboxOutput(SandboxOutputPayload),
    #[serde(rename = "sandbox.artifact")]
    SandboxArtifact(SandboxArtifactPayload),
    #[serde(rename = "sandbox.completed")]
    SandboxCompleted(SandboxCompletedPayload),
    #[serde(rename = "persona.state")]
    PersonaState(PersonaStatePayload),
    #[serde(rename = "workflow.started")]
    WorkflowStarted(WorkflowStartedPayload),
    #[serde(rename = "workflow.step")]
    WorkflowStep(WorkflowStepPayload),
    #[serde(rename = "workflow.test_result")]
    WorkflowTestResult(WorkflowTestResultPayload),
    #[serde(rename = "workflow.patch_proposed")]
    WorkflowPatchProposed(WorkflowPatchProposedPayload),
    #[serde(rename = "workflow.completed")]
    WorkflowCompleted(WorkflowCompletedPayload),
    #[serde(rename = "permission.requested")]
    PermissionRequested(PermissionRequestPayload),
    #[serde(rename = "permission.resolved")]
    PermissionResolved(PermissionResolvedPayload),
    #[serde(rename = "error.occurred")]
    ErrorOccurred(ErrorPayload),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct EventEnvelope {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    #[serde(flatten)]
    pub event: Event,
}

impl EventEnvelope {
    pub fn new(id: impl Into<String>, timestamp: DateTime<Utc>, event: Event) -> Self {
        Self {
            id: id.into(),
            timestamp,
            event,
        }
    }
}
