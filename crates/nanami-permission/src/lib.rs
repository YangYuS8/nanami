use std::collections::HashMap;

use nanami_protocol::{
    PermissionDecision, PermissionLevel, PermissionRequestPayload, PermissionResolvedPayload,
    PermissionScope,
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
}

impl PermissionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn request_permission(&mut self, request: PermissionRequestPayload) -> PermissionRequestPayload {
        request
    }

    pub fn resolve_permission(
        &mut self,
        permission_id: &str,
        decision: PermissionDecision,
    ) -> PermissionResolvedPayload {
        self.decisions
            .insert(permission_id.to_owned(), decision.clone());

        PermissionResolvedPayload {
            permission_id: permission_id.to_owned(),
            decision,
        }
    }

    pub fn decision_for(&self, permission_id: &str) -> Option<PermissionDecision> {
        self.decisions.get(permission_id).cloned()
    }

    pub fn classify_tool_request(
        &self,
        request: DangerousToolRequest,
    ) -> Option<PermissionRequestPayload> {
        let tool = request.tool.to_lowercase();
        let arguments = request.arguments.unwrap_or_default();

        let (level, action) = if matches_any(&tool, &["filesystem.read", "file.read", "read_file", "project.read"]) {
            (PermissionLevel::L2, "filesystem.read")
        } else if matches_any(&tool, &["filesystem.write", "file.write", "write_file", "apply_patch"]) {
            (PermissionLevel::L3, "filesystem.write")
        } else if matches_any(&tool, &["shell", "terminal", "command.run", "local.exec", "process.spawn"]) {
            (PermissionLevel::L4, "command.execute")
        } else if matches_any(&tool, &["sandbox.mount", "cubesandbox.mount"]) {
            (PermissionLevel::L5, "sandbox.mount")
        } else if matches_any(&tool, &["network.fetch", "http.request", "download", "dependency.install"]) {
            (PermissionLevel::L6, "network.access")
        } else if matches_any(&tool, &["delete_file", "filesystem.delete", "system.modify", "package.install", "service.modify"]) {
            (PermissionLevel::L7, if tool.contains("delete") { "filesystem.delete" } else { "system.modify" })
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
            target: sanitize_target(if arguments.is_empty() { &request.tool } else { &arguments }),
            reason: format!(
                "OpenClaw requested potentially dangerous tool: {}",
                request.tool
            ),
            scope: PermissionScope::Task,
            expires: "task_completed".into(),
        })
    }
}

fn matches_any(tool: &str, candidates: &[&str]) -> bool {
    candidates.iter().any(|candidate| tool == *candidate || tool.contains(candidate))
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
