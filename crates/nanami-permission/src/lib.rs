use std::collections::HashMap;

use nanami_protocol::{
    AuditAction, AuditRecord, PermissionDecision, PermissionLevel, PermissionRequestPayload,
    PermissionResolvedPayload, PermissionScope,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DangerousToolRequest {
    pub task_id: Option<String>,
    pub tool_call_id: String,
    pub tool: String,
    pub arguments: Option<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Default)]
pub struct PermissionManager {
    decisions: HashMap<String, PermissionDecision>,
    audit_records: Vec<AuditRecord>,
    next_audit_id: usize,
}

impl PermissionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn request_permission(
        &mut self,
        request: PermissionRequestPayload,
    ) -> PermissionRequestPayload {
        let audit_id = self.next_audit_id();
        self.audit_records.push(AuditRecord {
            audit_id,
            timestamp: chrono::Utc::now(),
            task_id: request.task_id.clone(),
            permission_id: request.permission_id.clone(),
            action: AuditAction::PermissionRequested,
            level: Some(request.level.clone()),
            permission_action: Some(request.action.clone()),
            target: Some(sanitize_target(&request.target)),
            decision: None,
            result: "recorded_only".into(),
        });

        request
    }

    pub fn resolve_permission(
        &mut self,
        permission_id: &str,
        decision: PermissionDecision,
    ) -> PermissionResolvedPayload {
        self.decisions
            .insert(permission_id.to_owned(), decision.clone());

        let audit_id = self.next_audit_id();
        self.audit_records.push(AuditRecord {
            audit_id,
            timestamp: chrono::Utc::now(),
            task_id: None,
            permission_id: permission_id.to_owned(),
            action: AuditAction::PermissionResolved,
            level: None,
            permission_action: None,
            target: None,
            decision: Some(decision.clone()),
            result: "recorded_only".into(),
        });

        PermissionResolvedPayload {
            permission_id: permission_id.to_owned(),
            decision,
        }
    }

    pub fn decision_for(&self, permission_id: &str) -> Option<PermissionDecision> {
        self.decisions.get(permission_id).cloned()
    }

    pub fn audit_records(&self) -> Vec<AuditRecord> {
        self.audit_records.clone()
    }

    pub fn classify_tool_request(
        &self,
        request: DangerousToolRequest,
    ) -> Option<PermissionRequestPayload> {
        let tool = request.tool.to_lowercase();
        let arguments = request.arguments.unwrap_or_default();

        let (level, action) = if matches_any(
            &tool,
            &["filesystem.read", "file.read", "read_file", "project.read"],
        ) {
            (PermissionLevel::L2, "filesystem.read")
        } else if matches_any(
            &tool,
            &[
                "filesystem.write",
                "file.write",
                "write_file",
                "apply_patch",
            ],
        ) {
            (PermissionLevel::L3, "filesystem.write")
        } else if matches_any(
            &tool,
            &[
                "shell",
                "terminal",
                "command.run",
                "local.exec",
                "process.spawn",
            ],
        ) {
            (PermissionLevel::L4, "command.execute")
        } else if matches_any(&tool, &["sandbox.mount", "cubesandbox.mount"]) {
            (PermissionLevel::L5, "sandbox.mount")
        } else if matches_any(
            &tool,
            &[
                "network.fetch",
                "http.request",
                "download",
                "dependency.install",
            ],
        ) {
            (PermissionLevel::L6, "network.access")
        } else if matches_any(
            &tool,
            &[
                "delete_file",
                "filesystem.delete",
                "system.modify",
                "package.install",
                "service.modify",
            ],
        ) {
            (
                PermissionLevel::L7,
                if tool.contains("delete") {
                    "filesystem.delete"
                } else {
                    "system.modify"
                },
            )
        } else if looks_dangerous(&tool, &arguments) {
            (PermissionLevel::L7, "system.modify")
        } else {
            return None;
        };

        Some(PermissionRequestPayload {
            task_id: request.task_id,
            permission_id: format!("perm_{}", request.tool_call_id),
            level,
            action: action.into(),
            target: sanitize_target(if arguments.is_empty() {
                &request.tool
            } else {
                &arguments
            }),
            reason: format!(
                "OpenClaw requested potentially dangerous tool: {}",
                request.tool
            ),
            scope: PermissionScope::Task,
            expires: "task_completed".into(),
        })
    }

    fn next_audit_id(&mut self) -> String {
        self.next_audit_id += 1;
        format!("audit_{:03}", self.next_audit_id)
    }
}

fn matches_any(tool: &str, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| tool == *candidate || tool.contains(candidate))
}

fn looks_dangerous(tool: &str, arguments: &str) -> bool {
    let haystack = format!("{} {}", tool, arguments).to_lowercase();
    haystack.contains("exec")
        || haystack.contains("command")
        || haystack.contains("delete")
        || haystack.contains("modify")
        || haystack.contains("install")
        || haystack.contains("sudo")
}

fn sanitize_target(target: &str) -> String {
    target
        .replace("authorization", "redacted")
        .replace("cookie", "redacted")
        .replace("token", "redacted")
}
