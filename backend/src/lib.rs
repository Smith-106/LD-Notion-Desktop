// LD-Notion Hub 后端 — 公共接口

#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]

pub mod config;
pub mod db;
pub mod engine;
pub mod mcp;
pub mod search;

use axum::{
    extract::{Path, Query, State},
    Json,
};
use axum::body::Bytes;
use rusqlite::params;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub config: config::Config,
}

// ── 请求体 ──

#[derive(Deserialize)]
pub struct CreateWorkspaceReq {
    pub name: String,
}

#[derive(Deserialize)]
pub struct RenameWorkspaceReq {
    pub name: String,
}

#[derive(Deserialize)]
pub struct CreatePageReq {
    pub workspace_id: String,
    pub parent_id: Option<String>,
    pub title: String,
    #[serde(default)]
    pub is_folder: bool,
}

#[derive(Deserialize)]
pub struct UpdatePageReq {
    pub body: String,
}

#[derive(Deserialize)]
pub struct RenamePageReq {
    pub title: String,
}

#[derive(Deserialize)]
pub struct MovePageReq {
    pub parent_id: Option<String>,
}

#[derive(Deserialize)]
pub struct ImportPageReq {
    pub workspace_id: String,
    pub parent_id: Option<String>,
    pub title: String,
    pub body: String,
}

#[derive(Deserialize)]
pub struct UpdateTagsReq {
    pub tags: Vec<String>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default = "default_limit")]
    pub limit: i32,
    #[serde(default)]
    pub offset: i32,
}

#[derive(Deserialize)]
pub struct RecentQuery {
    #[serde(default = "default_recent_limit")]
    pub limit: i32,
}

fn default_mode() -> String { "and".to_string() }
const fn default_limit() -> i32 { 20 }
const fn default_recent_limit() -> i32 { 10 }

// ── 健康检查 ──

#[allow(clippy::unused_async)]
pub async fn health_check() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

// ── 工作区 API ──

pub async fn list_workspaces(State(state): State<Arc<AppState>>) -> Json<Value> {
    let conn = state.db.lock().await;
    match engine::workspace::list(&conn) {
        Ok(list) => Json(json!({"ok": true, "data": list})),
        Err(e) => Json(json!({"ok": false, "error": e.to_string()})),
    }
}

pub async fn create_workspace(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateWorkspaceReq>,
) -> Json<Value> {
    let name = body.name.trim();
    if name.is_empty() {
        return Json(json!({"ok": false, "error": "工作区名称不能为空"}));
    }
    let conn = state.db.lock().await;
    match engine::workspace::create(&conn, name, &state.config.storage_root) {
        Ok(ws) => Json(json!({"ok": true, "data": ws})),
        Err(e) => Json(json!({"ok": false, "error": e.to_string()})),
    }
}

pub async fn delete_workspace(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    let conn = state.db.lock().await;
    match engine::workspace::delete(&conn, &id, &state.config.storage_root) {
        Ok(removed) => Json(json!({"ok": true, "removed": removed})),
        Err(e) => Json(json!({"ok": false, "error": e.to_string()})),
    }
}

pub async fn rename_workspace(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<RenameWorkspaceReq>,
) -> Json<Value> {
    let name = body.name.trim();
    if name.is_empty() {
        return Json(json!({"ok": false, "error": "工作区名称不能为空"}));
    }
    let conn = state.db.lock().await;
    match engine::workspace::rename(&conn, &id, name) {
        Ok(ws) => Json(json!({"ok": true, "data": ws})),
        Err(e) => Json(json!({"ok": false, "error": e.to_string()})),
    }
}

pub async fn workspace_stats(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<String>,
) -> Json<Value> {
    let conn = state.db.lock().await;
    let page_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pages WHERE workspace_id = ?1",
            [&ws_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    Json(json!({"ok": true, "data": {"page_count": page_count}}))
}

// ── 页面 API ──

pub async fn duplicate_page(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    let conn = state.db.lock().await;
    match engine::page::duplicate(&conn, &id, &state.config.storage_root) {
        Ok(page) => Json(json!({"ok": true, "data": page})),
        Err(e) => Json(json!({"ok": false, "error": e.to_string()})),
    }
}

pub async fn create_page(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreatePageReq>,
) -> Json<Value> {
    let title = body.title.trim();
    if title.is_empty() {
        return Json(json!({"ok": false, "error": "页面标题不能为空"}));
    }
    let conn = state.db.lock().await;
    let parent_id = body.parent_id.as_deref().filter(|s| !s.is_empty());
    let result = if body.is_folder {
        engine::page::create_folder(&conn, &body.workspace_id, parent_id, title)
    } else {
        engine::page::create(
            &conn,
            &body.workspace_id,
            parent_id,
            title,
            &state.config.storage_root,
        )
    };
    match result {
        Ok(page) => Json(json!({"ok": true, "data": page})),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("FOREIGN KEY") {
                Json(json!({"ok": false, "error": "工作区不存在"}))
            } else {
                Json(json!({"ok": false, "error": msg}))
            }
        }
    }
}

pub async fn get_page(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    let conn = state.db.lock().await;
    match engine::page::find(&conn, &id) {
        Ok(Some(page)) => Json(json!({"ok": true, "data": page})),
        Ok(None) => Json(json!({"ok": false, "error": "page not found"})),
        Err(e) => Json(json!({"ok": false, "error": e.to_string()})),
    }
}

pub async fn get_page_content(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    let conn = state.db.lock().await;
    match engine::page::read_content(&conn, &id, &state.config.storage_root) {
        Ok(Some(content)) => Json(json!({"ok": true, "data": content})),
        Ok(None) => Json(json!({"ok": false, "error": "page not found"})),
        Err(e) => Json(json!({"ok": false, "error": e.to_string()})),
    }
}

pub async fn update_page_content(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<UpdatePageReq>,
) -> Json<Value> {
    let conn = state.db.lock().await;
    match engine::page::update_content(&conn, &id, &body.body, &state.config.storage_root) {
        Ok(()) => {
            if let Ok(Some(content)) = engine::page::read_content(&conn, &id, &state.config.storage_root) {
                let _ = search::index_page(&conn, &id, &content.title, &content.body, &content.tags.join(", "));
            }
            Json(json!({"ok": true}))
        }
        Err(e) => Json(json!({"ok": false, "error": e.to_string()})),
    }
}

pub async fn delete_page(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    let conn = state.db.lock().await;
    match engine::trash::soft_delete(&conn, &id, &state.config.storage_root) {
        Ok(removed) => Json(json!({"ok": true, "removed": removed})),
        Err(e) => Json(json!({"ok": false, "error": e.to_string()})),
    }
}

pub async fn get_page_tree(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<String>,
) -> Json<Value> {
    let conn = state.db.lock().await;
    match engine::page_tree::get_tree(&conn, &ws_id, None) {
        Ok(tree) => Json(json!({"ok": true, "data": tree})),
        Err(e) => Json(json!({"ok": false, "error": e.to_string()})),
    }
}

// ── 搜索 API ──

pub async fn search_pages(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> Json<Value> {
    let conn = state.db.lock().await;
    match search::search(&conn, &params.q, &params.mode, params.limit, params.offset) {
        Ok(output) => Json(json!({"ok": true, "data": output})),
        Err(e) => Json(json!({"ok": false, "error": e.to_string()})),
    }
}

// ── 页面管理 API ──

pub async fn rename_page(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<RenamePageReq>,
) -> Json<Value> {
    let title = body.title.trim();
    if title.is_empty() {
        return Json(json!({"ok": false, "error": "页面标题不能为空"}));
    }
    let conn = state.db.lock().await;
    match engine::page::rename(&conn, &id, title, &state.config.storage_root) {
        Ok(page) => Json(json!({"ok": true, "data": page})),
        Err(e) => Json(json!({"ok": false, "error": e.to_string()})),
    }
}

pub async fn move_page(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<MovePageReq>,
) -> Json<Value> {
    let conn = state.db.lock().await;
    let parent_id = body.parent_id.as_deref().filter(|s| !s.is_empty());
    match engine::page::move_to(&conn, &id, parent_id) {
        Ok(page) => Json(json!({"ok": true, "data": page})),
        Err(e) => Json(json!({"ok": false, "error": e.to_string()})),
    }
}

// ── 导入 API ──

pub async fn import_page(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ImportPageReq>,
) -> Json<Value> {
    let title = body.title.trim();
    if title.is_empty() {
        return Json(json!({"ok": false, "error": "页面标题不能为空"}));
    }
    let conn = state.db.lock().await;
    let parent_id = body.parent_id.as_deref().filter(|s| !s.is_empty());
    match engine::page::create(&conn, &body.workspace_id, parent_id, title, &state.config.storage_root) {
        Ok(page) => {
            if !body.body.is_empty() {
                let _ = engine::page::update_content(&conn, &page.id, &body.body, &state.config.storage_root);
            }
            Json(json!({"ok": true, "data": page}))
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("FOREIGN KEY") {
                Json(json!({"ok": false, "error": "工作区不存在"}))
            } else {
                Json(json!({"ok": false, "error": msg}))
            }
        }
    }
}

// ── 标签 API ──

pub async fn update_tags(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<UpdateTagsReq>,
) -> Json<Value> {
    let conn = state.db.lock().await;
    match engine::page::update_tags(&conn, &id, &body.tags, &state.config.storage_root) {
        Ok(page) => Json(json!({"ok": true, "data": page})),
        Err(e) => Json(json!({"ok": false, "error": e.to_string()})),
    }
}

pub async fn list_tags(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<String>,
) -> Json<Value> {
    let conn = state.db.lock().await;
    match engine::page::list_tags(&conn, &ws_id, &state.config.storage_root) {
        Ok(tags) => {
            let tag_list: Vec<Value> = tags
                .into_iter()
                .map(|(name, count)| json!({"name": name, "count": count}))
                .collect();
            Json(json!({"ok": true, "data": tag_list}))
        }
        Err(e) => Json(json!({"ok": false, "error": e.to_string()})),
    }
}

// ── 最近编辑 & 收藏 API ──

pub async fn list_recent(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<String>,
    Query(params): Query<RecentQuery>,
) -> Json<Value> {
    let conn = state.db.lock().await;
    match engine::page::list_recent(&conn, &ws_id, params.limit) {
        Ok(pages) => Json(json!({"ok": true, "data": pages})),
        Err(e) => Json(json!({"ok": false, "error": e.to_string()})),
    }
}

pub async fn list_pinned(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<String>,
) -> Json<Value> {
    let conn = state.db.lock().await;
    match engine::page::list_pinned(&conn, &ws_id) {
        Ok(pages) => Json(json!({"ok": true, "data": pages})),
        Err(e) => Json(json!({"ok": false, "error": e.to_string()})),
    }
}

pub async fn toggle_pin(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    let conn = state.db.lock().await;
    match engine::page::toggle_pin(&conn, &id) {
        Ok(page) => Json(json!({"ok": true, "data": page})),
        Err(e) => Json(json!({"ok": false, "error": e.to_string()})),
    }
}

// ── 回收站 API ──

pub async fn list_trash(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<String>,
) -> Json<Value> {
    let conn = state.db.lock().await;
    match engine::trash::list(&conn, &ws_id) {
        Ok(items) => Json(json!({"ok": true, "data": items})),
        Err(e) => Json(json!({"ok": false, "error": e.to_string()})),
    }
}

pub async fn restore_page(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    let conn = state.db.lock().await;
    match engine::trash::restore(&conn, &id, &state.config.storage_root) {
        Ok(restored) => Json(json!({"ok": true, "restored": restored})),
        Err(e) => Json(json!({"ok": false, "error": e.to_string()})),
    }
}

pub async fn purge_page(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    let conn = state.db.lock().await;
    match engine::trash::purge(&conn, &id, &state.config.storage_root) {
        Ok(removed) => Json(json!({"ok": true, "removed": removed})),
        Err(e) => Json(json!({"ok": false, "error": e.to_string()})),
    }
}

pub async fn empty_trash(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<String>,
) -> Json<Value> {
    let conn = state.db.lock().await;
    match engine::trash::empty(&conn, &ws_id, &state.config.storage_root) {
        Ok(count) => Json(json!({"ok": true, "count": count})),
        Err(e) => Json(json!({"ok": false, "error": e.to_string()})),
    }
}

// ── 页面排序 API ──

#[derive(Deserialize)]
pub struct ReorderReq {
    pub page_ids: Vec<String>,
}

pub async fn reorder_pages(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ReorderReq>,
) -> Json<Value> {
    let conn = state.db.lock().await;
    for (i, id) in body.page_ids.iter().enumerate() {
        if let Err(e) = conn.execute(
            "UPDATE pages SET sort_order = ?1 WHERE id = ?2",
            params![(i + 1) as i32, id],
        ) {
            return Json(json!({"ok": false, "error": e.to_string()}));
        }
    }
    Json(json!({"ok": true}))
}

// ── 图片 API ──

use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};

#[derive(Deserialize)]
pub struct UploadQuery {
    pub filename: String,
}

pub async fn upload_image(
    State(state): State<Arc<AppState>>,
    Query(params): Query<UploadQuery>,
    body: Bytes,
) -> Json<Value> {
    match engine::image::save(&state.config.storage_root, &params.filename, &body) {
        Ok(path) => Json(json!({"ok": true, "path": path})),
        Err(e) => Json(json!({"ok": false, "error": e.to_string()})),
    }
}

pub async fn get_image(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Response {
    match engine::image::read(&state.config.storage_root, &path) {
        Ok((bytes, mime)) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, mime)],
            bytes,
        ).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain".to_string())],
            "not found",
        ).into_response(),
    }
}

// ── 备份 API ──

pub async fn export_workspace(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<String>,
) -> Response {
    let conn = state.db.lock().await;
    match engine::backup::export_workspace(&conn, &ws_id, &state.config.storage_root) {
        Ok(zip_data) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "application/zip".to_string()),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"workspace-backup.zip\"".to_string(),
                ),
            ],
            zip_data,
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "text/plain".to_string())],
            e.to_string(),
        ).into_response(),
    }
}

#[derive(Deserialize)]
pub struct ImportBackupReq {
    pub workspace_name: String,
}

pub async fn import_workspace(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ImportBackupReq>,
    body: Bytes,
) -> Json<Value> {
    let conn = state.db.lock().await;
    match engine::backup::import_workspace(
        &conn,
        &state.config.storage_root,
        &body,
        &params.workspace_name,
    ) {
        Ok(ws_id) => Json(json!({"ok": true, "workspace_id": ws_id})),
        Err(e) => Json(json!({"ok": false, "error": e.to_string()})),
    }
}
