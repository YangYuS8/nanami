use chrono::DateTime;
use nanami_protocol::{
    Event, EventEnvelope, PermissionDecision, PermissionLevel, PermissionRequestPayload,
    PermissionResolvedPayload, PermissionScope,
};

fn timestamp() -> chrono::DateTime<chrono::Utc> {
    DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
        .unwrap()
        .to_utc()
}

#[test]
fn permission_requested_serializes_json_shape() {
    let event = EventEnvelope::new(
        "evt_perm_req_001",
        timestamp(),
        Event::PermissionRequested(PermissionRequestPayload {
            task_id: Some("task_mock_001".into()),
            permission_id: "perm_mock_read_project".into(),
            level: PermissionLevel::L2,
            action: "filesystem.read".into(),
            target: "/home/user/Code/nanami".into(),
            reason: "Need to read project files for analysis".into(),
            scope: PermissionScope::Task,
            expires: "task_completed".into(),
        }),
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["type"], "permission.requested");
    assert_eq!(json["permission_id"], "perm_mock_read_project");
    assert_eq!(json["level"], "l2");
}

#[test]
fn permission_resolved_allow_once_serializes_json_shape() {
    let event = EventEnvelope::new(
        "evt_perm_res_001",
        timestamp(),
        Event::PermissionResolved(PermissionResolvedPayload {
            permission_id: "perm_mock_read_project".into(),
            decision: PermissionDecision::AllowOnce,
        }),
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["type"], "permission.resolved");
    assert_eq!(json["decision"], "allow_once");
}

#[test]
fn permission_resolved_allow_for_task_serializes_json_shape() {
    let event = EventEnvelope::new(
        "evt_perm_res_002",
        timestamp(),
        Event::PermissionResolved(PermissionResolvedPayload {
            permission_id: "perm_mock_read_project".into(),
            decision: PermissionDecision::AllowForTask,
        }),
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["decision"], "allow_for_task");
}

#[test]
fn permission_resolved_deny_serializes_json_shape() {
    let event = EventEnvelope::new(
        "evt_perm_res_003",
        timestamp(),
        Event::PermissionResolved(PermissionResolvedPayload {
            permission_id: "perm_mock_read_project".into(),
            decision: PermissionDecision::Deny,
        }),
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["decision"], "deny");
}

#[test]
fn permission_level_l4_serializes_snake_case() {
    let json = serde_json::to_value(PermissionLevel::L4).unwrap();

    assert_eq!(json, "l4");
}

#[test]
fn permission_decision_allow_for_task_serializes_snake_case() {
    let json = serde_json::to_value(PermissionDecision::AllowForTask).unwrap();

    assert_eq!(json, "allow_for_task");
}
