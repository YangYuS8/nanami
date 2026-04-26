use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use super::support::{
    body_json, request_manifest_preview_permission, resolve_permission, router,
    select_and_trust_project, temp_project_dir,
};
use crate::state::MANIFEST_PREVIEW_MAX_BYTES;

#[tokio::test]
async fn projects_current_manifest_preview_request_requires_selected_trusted_project() {
    let response = router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/current/manifest/preview-request")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn projects_current_manifest_preview_request_records_l2_permission_for_top_level_manifest() {
    let temp_dir = temp_project_dir("nanami_manifest_preview_request");
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::write(temp_dir.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();

    let app = router();
    let project_id = select_and_trust_project(&app, &temp_dir).await;

    let preview_request_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/current/manifest/preview-request")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = preview_request_response.status();
    let json = body_json(preview_request_response).await;

    let _ = std::fs::remove_dir_all(&temp_dir);

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["level"], "l2");
    assert_eq!(json["action"], "filesystem.read");
    assert_eq!(
        json["permission_id"],
        format!("perm_manifest_preview_{}", project_id)
    );
    assert_eq!(
        json["target"],
        temp_dir.join("Cargo.toml").display().to_string()
    );
}

#[tokio::test]
async fn projects_current_manifest_preview_requires_permission_decision() {
    let temp_dir = temp_project_dir("nanami_manifest_preview_permission");
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::write(temp_dir.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();

    let app = router();
    let _project_id = select_and_trust_project(&app, &temp_dir).await;

    let preview_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/projects/current/manifest/preview")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let _ = std::fs::remove_dir_all(&temp_dir);

    assert_eq!(preview_response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn projects_current_manifest_preview_returns_top_level_preview_after_allow_once() {
    let temp_dir = temp_project_dir("nanami_manifest_preview_allow");
    std::fs::create_dir_all(temp_dir.join("nested")).unwrap();
    std::fs::write(temp_dir.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();
    std::fs::write(
        temp_dir.join("nested").join("Cargo.toml"),
        "[package]\nname = \"nested\"\n",
    )
    .unwrap();

    let app = router();
    let project_id = select_and_trust_project(&app, &temp_dir).await;
    let permission_id = format!("perm_manifest_preview_{}", project_id);

    request_manifest_preview_permission(&app).await;
    resolve_permission(&app, &permission_id, "allow_once").await;

    let preview_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/projects/current/manifest/preview")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = preview_response.status();
    let json = body_json(preview_response).await;

    let _ = std::fs::remove_dir_all(&temp_dir);

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["project_id"], project_id);
    assert_eq!(json["kind"], "rust");
    assert_eq!(
        json["manifest_path"],
        temp_dir.join("Cargo.toml").display().to_string()
    );
    assert_eq!(json["content_preview"], "[package]\nname = \"demo\"\n");
    assert_eq!(json["truncated"], false);
    assert_eq!(json["size_bytes"], 24);
}

#[tokio::test]
async fn projects_current_manifest_preview_truncates_to_8kb() {
    let temp_dir = temp_project_dir("nanami_manifest_preview_truncate");
    std::fs::create_dir_all(&temp_dir).unwrap();
    let manifest_content = "a".repeat((MANIFEST_PREVIEW_MAX_BYTES as usize) + 17);
    std::fs::write(temp_dir.join("Cargo.toml"), &manifest_content).unwrap();

    let app = router();
    let project_id = select_and_trust_project(&app, &temp_dir).await;
    let permission_id = format!("perm_manifest_preview_{}", project_id);

    request_manifest_preview_permission(&app).await;
    resolve_permission(&app, &permission_id, "allow_for_task").await;

    let preview_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/projects/current/manifest/preview")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let json = body_json(preview_response).await;

    let _ = std::fs::remove_dir_all(&temp_dir);

    assert_eq!(json["truncated"], true);
    assert_eq!(json["size_bytes"], MANIFEST_PREVIEW_MAX_BYTES + 17);
    assert_eq!(
        json["content_preview"].as_str().unwrap().len(),
        MANIFEST_PREVIEW_MAX_BYTES as usize
    );
}

#[tokio::test]
async fn projects_current_manifest_summary_requires_permission_decision() {
    let temp_dir = temp_project_dir("nanami_manifest_summary_permission");
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::write(temp_dir.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();

    let app = router();
    let _project_id = select_and_trust_project(&app, &temp_dir).await;

    let summary_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/projects/current/manifest/summary")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let _ = std::fs::remove_dir_all(&temp_dir);

    assert_eq!(summary_response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn projects_current_manifest_summary_extracts_rust_fields_after_allow_once() {
    let temp_dir = temp_project_dir("nanami_manifest_summary_rust");
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::write(
        temp_dir.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n\n[dependencies]\nserde = \"1\"\ntokio = \"1\"\n",
    )
    .unwrap();

    let app = router();
    let project_id = select_and_trust_project(&app, &temp_dir).await;
    let permission_id = format!("perm_manifest_preview_{}", project_id);

    request_manifest_preview_permission(&app).await;
    resolve_permission(&app, &permission_id, "allow_once").await;

    let summary_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/projects/current/manifest/summary")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = summary_response.status();
    let json = body_json(summary_response).await;

    let _ = std::fs::remove_dir_all(&temp_dir);

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["kind"], "rust");
    assert_eq!(json["package_name"], "demo");
    assert_eq!(json["package_version"], "0.1.0");
    assert_eq!(json["dependency_count"], 2);
    assert!(json["script_count"].is_null());
}

#[tokio::test]
async fn projects_current_manifest_summary_extracts_node_fields_with_scripts_and_dependencies() {
    let temp_dir = temp_project_dir("nanami_manifest_summary_node");
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::write(
        temp_dir.join("package.json"),
        r#"{"name":"demo-node","version":"1.2.3","dependencies":{"react":"18"},"devDependencies":{"vite":"5","typescript":"5"},"scripts":{"dev":"vite","build":"vite build"}}"#,
    )
    .unwrap();

    let app = router();
    let project_id = select_and_trust_project(&app, &temp_dir).await;
    let permission_id = format!("perm_manifest_preview_{}", project_id);

    request_manifest_preview_permission(&app).await;
    resolve_permission(&app, &permission_id, "allow_for_task").await;

    let summary_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/projects/current/manifest/summary")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let json = body_json(summary_response).await;

    let _ = std::fs::remove_dir_all(&temp_dir);

    assert_eq!(json["kind"], "node");
    assert_eq!(json["package_name"], "demo-node");
    assert_eq!(json["package_version"], "1.2.3");
    assert_eq!(json["dependency_count"], 3);
    assert_eq!(json["script_count"], 2);
}

#[tokio::test]
async fn projects_current_manifest_summary_extracts_python_fields_and_tolerates_parse_failure() {
    let temp_dir = temp_project_dir("nanami_manifest_summary_python");
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::write(
        temp_dir.join("pyproject.toml"),
        "[project]\nname = \"demo-py\"\nversion = \"0.2.0\"\ndependencies = [\"fastapi\", \"uvicorn\"]\n",
    )
    .unwrap();

    let app = router();
    let project_id = select_and_trust_project(&app, &temp_dir).await;
    let permission_id = format!("perm_manifest_preview_{}", project_id);

    request_manifest_preview_permission(&app).await;
    resolve_permission(&app, &permission_id, "allow_for_task").await;

    let summary_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/projects/current/manifest/summary")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = summary_response.status();
    let json = body_json(summary_response).await;

    std::fs::write(temp_dir.join("pyproject.toml"), "not valid toml = [").unwrap();

    let fallback_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/projects/current/manifest/summary")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let fallback_json = body_json(fallback_response).await;

    let _ = std::fs::remove_dir_all(&temp_dir);

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["kind"], "python");
    assert_eq!(json["package_name"], "demo-py");
    assert_eq!(json["package_version"], "0.2.0");
    assert_eq!(json["dependency_count"], 2);
    assert!(json["script_count"].is_null());

    assert_eq!(fallback_json["kind"], "python");
    assert!(fallback_json["package_name"].is_null());
    assert!(fallback_json["package_version"].is_null());
    assert!(fallback_json["dependency_count"].is_null());
    assert_eq!(
        fallback_json["summary_text"],
        "Manifest summary unavailable"
    );
}
