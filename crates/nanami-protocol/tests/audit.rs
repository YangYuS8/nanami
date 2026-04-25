use chrono::DateTime;
use nanami_protocol::{
    AuditAction, AuditRecord, PermissionAuditLogResponse, PermissionDecision,
    PermissionDecisionStatus, PermissionLevel,
};

fn timestamp() -> chrono::DateTime<chrono::Utc> {
    DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
        .unwrap()
        .to_utc()
}

#[test]
fn audit_record_permission_requested_serializes_json_shape() {
    let record = AuditRecord {
        audit_id: "audit_001".into(),
        timestamp: timestamp(),
        task_id: Some("task_mock_001".into()),
        permission_id: "perm_mock_read_project".into(),
        action: AuditAction::PermissionRequested,
        level: Some(PermissionLevel::L2),
        permission_action: Some("filesystem.read".into()),
        target: Some("/home/user/Code/nanami".into()),
        decision: None,
        result: "recorded_only".into(),
    };

    let json = serde_json::to_value(record).unwrap();

    assert_eq!(json["action"], "permission_requested");
    assert_eq!(json["level"], "l2");
    assert_eq!(json["permission_action"], "filesystem.read");
}

#[test]
fn audit_record_permission_resolved_allow_once_serializes_json_shape() {
    let record = AuditRecord {
        audit_id: "audit_002".into(),
        timestamp: timestamp(),
        task_id: Some("task_mock_001".into()),
        permission_id: "perm_mock_read_project".into(),
        action: AuditAction::PermissionResolved,
        level: None,
        permission_action: None,
        target: Some("/home/user/Code/nanami".into()),
        decision: Some(PermissionDecision::AllowOnce),
        result: "recorded_only".into(),
    };

    let json = serde_json::to_value(record).unwrap();

    assert_eq!(json["action"], "permission_resolved");
    assert_eq!(json["decision"], "allow_once");
}

#[test]
fn permission_decision_status_with_decision_serializes_json_shape() {
    let status = PermissionDecisionStatus {
        permission_id: "perm_mock_read_project".into(),
        decision: Some(PermissionDecision::AllowForTask),
    };

    let json = serde_json::to_value(status).unwrap();

    assert_eq!(json["permission_id"], "perm_mock_read_project");
    assert_eq!(json["decision"], "allow_for_task");
}

#[test]
fn permission_decision_status_with_none_serializes_json_shape() {
    let status = PermissionDecisionStatus {
        permission_id: "perm_missing".into(),
        decision: None,
    };

    let json = serde_json::to_value(status).unwrap();

    assert!(json["decision"].is_null());
}

#[test]
fn permission_audit_log_response_serializes_records() {
    let response = PermissionAuditLogResponse {
        records: vec![AuditRecord {
            audit_id: "audit_003".into(),
            timestamp: timestamp(),
            task_id: None,
            permission_id: "perm_mock_read_project".into(),
            action: AuditAction::PermissionRequested,
            level: Some(PermissionLevel::L2),
            permission_action: Some("filesystem.read".into()),
            target: Some("/home/user/Code/nanami".into()),
            decision: None,
            result: "recorded_only".into(),
        }],
    };

    let json = serde_json::to_value(response).unwrap();

    assert!(json["records"].is_array());
    assert_eq!(json["records"][0]["audit_id"], "audit_003");
}
