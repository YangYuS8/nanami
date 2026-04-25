use nanami_protocol::{
    Event, EventEnvelope, SandboxArtifactPayload, SandboxCompletedPayload, SandboxMountMode,
    SandboxMountPayload, SandboxNetworkPolicy, SandboxStartedPayload, SandboxStatus,
    SandboxUpdatedPayload, ToolOutputPayload, ToolOutputStream,
};

pub fn mock_sandbox_events() -> Vec<EventEnvelope> {
    vec![
        EventEnvelope::new(
            "evt_sandbox_mock_started_001",
            chrono::Utc::now(),
            Event::SandboxStarted(SandboxStartedPayload {
                sandbox_id: "sandbox_mock_001".into(),
                task_id: "task_sandbox_mock_001".into(),
                template_id: "rust-workspace".into(),
                status: SandboxStatus::Starting,
                network_policy: SandboxNetworkPolicy::Disabled,
                mounts: vec![SandboxMountPayload {
                    host_path: "/mock/host/project".into(),
                    sandbox_path: "/workspace/project".into(),
                    mode: SandboxMountMode::ReadOnly,
                }],
            }),
        ),
        EventEnvelope::new(
            "evt_sandbox_mock_updated_001",
            chrono::Utc::now(),
            Event::SandboxUpdated(SandboxUpdatedPayload {
                sandbox_id: "sandbox_mock_001".into(),
                task_id: "task_sandbox_mock_001".into(),
                status: SandboxStatus::Running,
                summary: Some("Mock sandbox running".into()),
            }),
        ),
        EventEnvelope::new(
            "evt_sandbox_mock_stdout_001",
            chrono::Utc::now(),
            Event::SandboxOutput(ToolOutputPayload {
                task_id: "task_sandbox_mock_001".into(),
                tool_call_id: "sandbox_mock_001".into(),
                stream: ToolOutputStream::Stdout,
                content: "Checking workspace inside mock sandbox...".into(),
            }),
        ),
        EventEnvelope::new(
            "evt_sandbox_mock_stderr_001",
            chrono::Utc::now(),
            Event::SandboxOutput(ToolOutputPayload {
                task_id: "task_sandbox_mock_001".into(),
                tool_call_id: "sandbox_mock_001".into(),
                stream: ToolOutputStream::Stderr,
                content: "warning: mock sandbox stderr stream".into(),
            }),
        ),
        EventEnvelope::new(
            "evt_sandbox_mock_artifact_001",
            chrono::Utc::now(),
            Event::SandboxArtifact(SandboxArtifactPayload {
                sandbox_id: "sandbox_mock_001".into(),
                task_id: "task_sandbox_mock_001".into(),
                name: "mock-report.txt".into(),
                path: "/workspace/output/mock-report.txt".into(),
                media_type: "text/plain".into(),
                size_bytes: 128,
            }),
        ),
        EventEnvelope::new(
            "evt_sandbox_mock_completed_001",
            chrono::Utc::now(),
            Event::SandboxCompleted(SandboxCompletedPayload {
                sandbox_id: "sandbox_mock_001".into(),
                task_id: "task_sandbox_mock_001".into(),
                status: SandboxStatus::Completed,
                exit_code: Some(0),
                summary: Some("Mock sandbox completed without real execution".into()),
            }),
        ),
    ]
}

#[cfg(test)]
mod tests {
    #[test]
    fn mock_sandbox_events_returns_started_running_output_artifact_completed_sequence() {
        let events = crate::mock_sandbox_events();

        assert_eq!(events.len(), 6);
        let json = serde_json::to_value(&events).unwrap();
        let items = json.as_array().unwrap();

        assert_eq!(items[0]["type"], "sandbox.started");
        assert_eq!(items[1]["type"], "sandbox.updated");
        assert_eq!(items[2]["type"], "sandbox.output");
        assert_eq!(items[3]["type"], "sandbox.output");
        assert_eq!(items[4]["type"], "sandbox.artifact");
        assert_eq!(items[5]["type"], "sandbox.completed");
    }
}
