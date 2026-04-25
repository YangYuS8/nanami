use std::collections::HashMap;

use nanami_protocol::{PermissionDecision, PermissionRequestPayload, PermissionResolvedPayload};

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
}
