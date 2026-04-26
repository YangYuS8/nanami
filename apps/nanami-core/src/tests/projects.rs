use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use super::support::{body_json, router, select_and_trust_project, temp_project_dir};

#[tokio::test]
async fn projects_mock_current_returns_mock_project_metadata() {
    let response = router()
        .oneshot(
            Request::builder()
                .uri("/projects/mock/current")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let json = body_json(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["project_id"], "project_mock_001");
    assert_eq!(json["display_name"], "Nanami Mock Workspace");
    assert_eq!(json["project_path"], "/mock/project");
    assert_eq!(json["kind"], "rust");
    assert_eq!(json["trust_status"], "trusted_mock");
}

#[tokio::test]
async fn projects_select_detects_top_level_rust_manifest_and_returns_selected_untrusted() {
    let temp_dir = temp_project_dir("nanami_project_select_rust");
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::write(temp_dir.join("Cargo.toml"), "").unwrap();

    let response = router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/select")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"project_path":"{}"}}"#,
                    temp_dir.display()
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let json = body_json(response).await;

    let _ = std::fs::remove_dir_all(&temp_dir);

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["kind"], "rust");
    assert_eq!(json["trust_status"], "selected_untrusted");
}

#[tokio::test]
async fn projects_select_returns_unknown_when_no_top_level_manifest_exists() {
    let temp_dir = temp_project_dir("nanami_project_select_unknown");
    std::fs::create_dir_all(temp_dir.join("nested")).unwrap();
    std::fs::write(temp_dir.join("nested").join("Cargo.toml"), "").unwrap();

    let response = router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/select")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"project_path":"{}"}}"#,
                    temp_dir.display()
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    let json = body_json(response).await;

    let _ = std::fs::remove_dir_all(&temp_dir);

    assert_eq!(json["kind"], "unknown");
    assert_eq!(json["trust_status"], "selected_untrusted");
}

#[tokio::test]
async fn projects_trust_updates_selected_project_to_selected_trusted() {
    let temp_dir = temp_project_dir("nanami_project_trust");
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::write(temp_dir.join("Cargo.toml"), "").unwrap();

    let app = router();
    let select_json = super::support::select_project(&app, &temp_dir).await;
    let project_id = select_json["project_id"].as_str().unwrap().to_owned();

    let trust_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/trust")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"project_id":"{}"}}"#, project_id)))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = trust_response.status();
    let trust_json = body_json(trust_response).await;

    let _ = std::fs::remove_dir_all(&temp_dir);

    assert_eq!(status, StatusCode::OK);
    assert_eq!(trust_json["trust_status"], "selected_trusted");
}

#[tokio::test]
async fn projects_trust_rejects_non_selected_project_id() {
    let response = router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/trust")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"project_id":"project_missing_001"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn projects_current_structure_requires_selected_trusted_project() {
    let response = router()
        .oneshot(
            Request::builder()
                .uri("/projects/current/structure")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn projects_current_structure_returns_shallow_summary_for_selected_trusted_project() {
    let temp_dir = temp_project_dir("nanami_project_structure");
    std::fs::create_dir_all(temp_dir.join("src")).unwrap();
    std::fs::write(temp_dir.join("Cargo.toml"), "").unwrap();
    std::fs::write(temp_dir.join("README.md"), "").unwrap();
    std::fs::write(temp_dir.join(".gitignore"), "").unwrap();
    std::fs::write(temp_dir.join("src").join("nested.rs"), "").unwrap();

    let app = router();
    let project_id = select_and_trust_project(&app, &temp_dir).await;

    let structure_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/projects/current/structure")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = structure_response.status();
    let json = body_json(structure_response).await;

    let _ = std::fs::remove_dir_all(&temp_dir);

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["project_id"], project_id);
    assert!(
        json["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["name"] == "Cargo.toml"
                && entry["entry_type"] == "file"
                && entry["marker"] == "manifest")
    );
    assert!(
        json["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["name"] == "src"
                && entry["entry_type"] == "directory"
                && entry["marker"] == "source_dir")
    );
    assert!(
        json["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["name"] == ".gitignore" && entry["marker"] == "config")
    );
    assert!(
        json["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["name"] == "README.md" && entry["marker"] == "other")
    );
    assert!(
        !json["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["relative_path"] == "src/nested.rs")
    );
}
