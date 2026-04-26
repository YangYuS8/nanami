use nanami_protocol::{
    ManifestPreview, ManifestSummary, ProjectKind, ProjectMetadata, ProjectStructureEntry,
    ProjectStructureEntryType, ProjectStructureMarker, ProjectStructureSummary, ProjectTrustStatus,
};

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
fn project_trust_status_selected_trusted_serializes_snake_case() {
    let json = serde_json::to_value(ProjectTrustStatus::SelectedTrusted).unwrap();

    assert_eq!(json, "selected_trusted");
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

#[test]
fn project_structure_entry_type_file_serializes_snake_case() {
    let json = serde_json::to_value(ProjectStructureEntryType::File).unwrap();

    assert_eq!(json, "file");
}

#[test]
fn project_structure_entry_type_directory_serializes_snake_case() {
    let json = serde_json::to_value(ProjectStructureEntryType::Directory).unwrap();

    assert_eq!(json, "directory");
}

#[test]
fn project_structure_marker_manifest_serializes_snake_case() {
    let json = serde_json::to_value(ProjectStructureMarker::Manifest).unwrap();

    assert_eq!(json, "manifest");
}

#[test]
fn project_structure_marker_source_dir_serializes_snake_case() {
    let json = serde_json::to_value(ProjectStructureMarker::SourceDir).unwrap();

    assert_eq!(json, "source_dir");
}

#[test]
fn project_structure_marker_config_serializes_snake_case() {
    let json = serde_json::to_value(ProjectStructureMarker::Config).unwrap();

    assert_eq!(json, "config");
}

#[test]
fn project_structure_marker_other_serializes_snake_case() {
    let json = serde_json::to_value(ProjectStructureMarker::Other).unwrap();

    assert_eq!(json, "other");
}

#[test]
fn project_structure_summary_serializes_json_shape() {
    let summary = ProjectStructureSummary {
        project_id: "project_selected_demo".into(),
        project_path: "/mock/project".into(),
        entries: vec![
            ProjectStructureEntry {
                name: "Cargo.toml".into(),
                relative_path: "Cargo.toml".into(),
                entry_type: ProjectStructureEntryType::File,
                marker: ProjectStructureMarker::Manifest,
            },
            ProjectStructureEntry {
                name: "src".into(),
                relative_path: "src".into(),
                entry_type: ProjectStructureEntryType::Directory,
                marker: ProjectStructureMarker::SourceDir,
            },
            ProjectStructureEntry {
                name: ".gitignore".into(),
                relative_path: ".gitignore".into(),
                entry_type: ProjectStructureEntryType::File,
                marker: ProjectStructureMarker::Config,
            },
            ProjectStructureEntry {
                name: "README.md".into(),
                relative_path: "README.md".into(),
                entry_type: ProjectStructureEntryType::File,
                marker: ProjectStructureMarker::Other,
            },
        ],
    };

    let json = serde_json::to_value(summary).unwrap();

    assert_eq!(json["project_id"], "project_selected_demo");
    assert_eq!(json["project_path"], "/mock/project");
    assert_eq!(json["entries"][0]["name"], "Cargo.toml");
    assert_eq!(json["entries"][0]["entry_type"], "file");
    assert_eq!(json["entries"][0]["marker"], "manifest");
    assert_eq!(json["entries"][1]["name"], "src");
    assert_eq!(json["entries"][1]["entry_type"], "directory");
    assert_eq!(json["entries"][1]["marker"], "source_dir");
    assert_eq!(json["entries"][2]["marker"], "config");
    assert_eq!(json["entries"][3]["marker"], "other");
}

#[test]
fn manifest_preview_serializes_json_shape() {
    let preview = ManifestPreview {
        project_id: "project_selected_demo".into(),
        manifest_path: "/mock/project/Cargo.toml".into(),
        kind: ProjectKind::Rust,
        content_preview: "[package]\nname = \"demo\"\n".into(),
        truncated: false,
        size_bytes: 26,
    };

    let json = serde_json::to_value(preview).unwrap();

    assert_eq!(json["project_id"], "project_selected_demo");
    assert_eq!(json["manifest_path"], "/mock/project/Cargo.toml");
    assert_eq!(json["kind"], "rust");
    assert_eq!(json["content_preview"], "[package]\nname = \"demo\"\n");
    assert_eq!(json["truncated"], false);
    assert_eq!(json["size_bytes"], 26);
}

#[test]
fn manifest_summary_serializes_json_shape() {
    let summary = ManifestSummary {
        project_id: "project_selected_demo".into(),
        manifest_path: "/mock/project/Cargo.toml".into(),
        kind: ProjectKind::Rust,
        package_name: Some("demo".into()),
        package_version: Some("0.1.0".into()),
        dependency_count: Some(2),
        script_count: None,
        summary_text: "Rust package demo 0.1.0 with 2 dependencies".into(),
    };

    let json = serde_json::to_value(summary).unwrap();

    assert_eq!(json["project_id"], "project_selected_demo");
    assert_eq!(json["manifest_path"], "/mock/project/Cargo.toml");
    assert_eq!(json["kind"], "rust");
    assert_eq!(json["package_name"], "demo");
    assert_eq!(json["package_version"], "0.1.0");
    assert_eq!(json["dependency_count"], 2);
    assert!(json["script_count"].is_null());
    assert_eq!(
        json["summary_text"],
        "Rust package demo 0.1.0 with 2 dependencies"
    );
}
