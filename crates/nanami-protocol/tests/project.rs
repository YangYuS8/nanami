use nanami_protocol::{ProjectKind, ProjectMetadata, ProjectTrustStatus};

#[test]
fn project_kind_rust_serializes_snake_case() {
    let json = serde_json::to_value(ProjectKind::Rust).unwrap();

    assert_eq!(json, "rust");
}

#[test]
fn project_trust_status_trusted_mock_serializes_snake_case() {
    let json = serde_json::to_value(ProjectTrustStatus::TrustedMock).unwrap();

    assert_eq!(json, "trusted_mock");
}

#[test]
fn project_trust_status_selected_untrusted_serializes_snake_case() {
    let json = serde_json::to_value(ProjectTrustStatus::SelectedUntrusted).unwrap();

    assert_eq!(json, "selected_untrusted");
}

#[test]
fn project_metadata_serializes_json_shape() {
    let metadata = ProjectMetadata {
        project_id: "project_mock_001".into(),
        display_name: "Nanami Mock Workspace".into(),
        project_path: "/mock/project".into(),
        kind: ProjectKind::Rust,
        trust_status: ProjectTrustStatus::TrustedMock,
    };

    let json = serde_json::to_value(metadata).unwrap();

    assert_eq!(json["project_id"], "project_mock_001");
    assert_eq!(json["display_name"], "Nanami Mock Workspace");
    assert_eq!(json["project_path"], "/mock/project");
    assert_eq!(json["kind"], "rust");
    assert_eq!(json["trust_status"], "trusted_mock");
}
