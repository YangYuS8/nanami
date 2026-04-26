use nanami_protocol::{
    ProjectStructureEntry, ProjectStructureEntryType, ProjectStructureMarker,
    ProjectStructureSummary,
};

#[test]
fn project_structure_entry_type_directory_serializes_snake_case() {
    let json = serde_json::to_value(ProjectStructureEntryType::Directory).unwrap();

    assert_eq!(json, "directory");
}

#[test]
fn project_structure_marker_source_dir_serializes_snake_case() {
    let json = serde_json::to_value(ProjectStructureMarker::SourceDir).unwrap();

    assert_eq!(json, "source_dir");
}

#[test]
fn project_structure_summary_serializes_json_shape() {
    let summary = ProjectStructureSummary {
        project_id: "project_selected_demo".into(),
        project_path: "/mock/project".into(),
        entries: vec![ProjectStructureEntry {
            name: "src".into(),
            relative_path: "src".into(),
            entry_type: ProjectStructureEntryType::Directory,
            marker: ProjectStructureMarker::SourceDir,
        }],
    };

    let json = serde_json::to_value(summary).unwrap();

    assert_eq!(json["project_id"], "project_selected_demo");
    assert_eq!(json["project_path"], "/mock/project");
    assert_eq!(json["entries"][0]["name"], "src");
    assert_eq!(json["entries"][0]["entry_type"], "directory");
    assert_eq!(json["entries"][0]["marker"], "source_dir");
}
