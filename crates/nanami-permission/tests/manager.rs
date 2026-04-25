use nanami_permission::PermissionManager;
use nanami_protocol::{
    PermissionDecision, PermissionLevel, PermissionRequestPayload, PermissionScope,
};

use nanami_permission::DangerousToolRequest;

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

    let resolved =
        manager.resolve_permission(&request.permission_id, PermissionDecision::AllowOnce);

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

    let resolved =
        manager.resolve_permission(&request.permission_id, PermissionDecision::AllowForTask);

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

#[test]
fn classify_read_file_as_l2() {
    let manager = PermissionManager::new();

    let permission = manager.classify_tool_request(DangerousToolRequest {
        task_id: Some("task_001".into()),
        tool_call_id: "tool_001".into(),
        tool: "read_file".into(),
        arguments: Some("/workspace/project/src/main.rs".into()),
        summary: None,
    });

    assert_eq!(permission.unwrap().level, PermissionLevel::L2);
}

#[test]
fn classify_write_file_as_l3() {
    let manager = PermissionManager::new();

    let permission = manager.classify_tool_request(DangerousToolRequest {
        task_id: Some("task_001".into()),
        tool_call_id: "tool_002".into(),
        tool: "apply_patch".into(),
        arguments: Some("src/lib.rs".into()),
        summary: None,
    });

    assert_eq!(permission.unwrap().level, PermissionLevel::L3);
}

#[test]
fn classify_command_as_l4() {
    let manager = PermissionManager::new();

    let permission = manager.classify_tool_request(DangerousToolRequest {
        task_id: Some("task_001".into()),
        tool_call_id: "tool_003".into(),
        tool: "command.run".into(),
        arguments: Some("cargo check".into()),
        summary: None,
    });

    assert_eq!(permission.unwrap().level, PermissionLevel::L4);
}

#[test]
fn classify_sandbox_mount_as_l5() {
    let manager = PermissionManager::new();

    let permission = manager.classify_tool_request(DangerousToolRequest {
        task_id: Some("task_001".into()),
        tool_call_id: "tool_004".into(),
        tool: "sandbox.mount".into(),
        arguments: Some("/home/user/project".into()),
        summary: None,
    });

    assert_eq!(permission.unwrap().level, PermissionLevel::L5);
}

#[test]
fn classify_network_as_l6() {
    let manager = PermissionManager::new();

    let permission = manager.classify_tool_request(DangerousToolRequest {
        task_id: Some("task_001".into()),
        tool_call_id: "tool_005".into(),
        tool: "http.request".into(),
        arguments: Some("https://example.com".into()),
        summary: None,
    });

    assert_eq!(permission.unwrap().level, PermissionLevel::L6);
}

#[test]
fn classify_destructive_as_l7() {
    let manager = PermissionManager::new();

    let permission = manager.classify_tool_request(DangerousToolRequest {
        task_id: Some("task_001".into()),
        tool_call_id: "tool_006".into(),
        tool: "delete_file".into(),
        arguments: Some("/workspace/project/target".into()),
        summary: None,
    });

    assert_eq!(permission.unwrap().level, PermissionLevel::L7);
}

#[test]
fn classify_unknown_dangerous_tool_as_l7() {
    let manager = PermissionManager::new();

    let permission = manager.classify_tool_request(DangerousToolRequest {
        task_id: Some("task_001".into()),
        tool_call_id: "tool_007".into(),
        tool: "weird.exec".into(),
        arguments: Some("sudo rm -rf /".into()),
        summary: None,
    });

    assert_eq!(permission.unwrap().level, PermissionLevel::L7);
}

#[test]
fn classify_harmless_tool_as_none() {
    let manager = PermissionManager::new();

    let permission = manager.classify_tool_request(DangerousToolRequest {
        task_id: Some("task_001".into()),
        tool_call_id: "tool_008".into(),
        tool: "display.message".into(),
        arguments: None,
        summary: Some("show status".into()),
    });

    assert!(permission.is_none());
}
