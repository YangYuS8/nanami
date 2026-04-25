use nanami_permission::PermissionManager;
use nanami_protocol::{
    PermissionDecision, PermissionLevel, PermissionRequestPayload, PermissionScope,
};

fn mock_request() -> PermissionRequestPayload {
    PermissionRequestPayload {
        task_id: Some("task_mock_001".into()),
        permission_id: "perm_mock_read_project".into(),
        level: PermissionLevel::L2,
        action: "filesystem.read".into(),
        target: "/home/user/Code/nanami".into(),
        reason: "Need to read project files for analysis".into(),
        scope: PermissionScope::Task,
        expires: "task_completed".into(),
    }
}

#[test]
fn default_has_no_decision() {
    let manager = PermissionManager::new();

    assert_eq!(manager.decision_for("perm_missing"), None);
}

#[test]
fn resolve_allow_once_records_decision() {
    let mut manager = PermissionManager::new();
    let request = manager.request_permission(mock_request());

    let resolved = manager.resolve_permission(&request.permission_id, PermissionDecision::AllowOnce);

    assert_eq!(resolved.decision, PermissionDecision::AllowOnce);
    assert_eq!(
        manager.decision_for(&request.permission_id),
        Some(PermissionDecision::AllowOnce)
    );
}

#[test]
fn resolve_allow_for_task_records_decision() {
    let mut manager = PermissionManager::new();
    let request = manager.request_permission(mock_request());

    let resolved = manager.resolve_permission(&request.permission_id, PermissionDecision::AllowForTask);

    assert_eq!(resolved.decision, PermissionDecision::AllowForTask);
    assert_eq!(
        manager.decision_for(&request.permission_id),
        Some(PermissionDecision::AllowForTask)
    );
}

#[test]
fn resolve_deny_records_decision() {
    let mut manager = PermissionManager::new();
    let request = manager.request_permission(mock_request());

    let resolved = manager.resolve_permission(&request.permission_id, PermissionDecision::Deny);

    assert_eq!(resolved.decision, PermissionDecision::Deny);
    assert_eq!(
        manager.decision_for(&request.permission_id),
        Some(PermissionDecision::Deny)
    );
}
