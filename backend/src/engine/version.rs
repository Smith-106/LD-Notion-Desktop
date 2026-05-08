// 页面版本历史

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::Page;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageVersion {
    pub id: String,
    pub page_id: String,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    pub created_at: String,
}

const MAX_VERSIONS_PER_PAGE: i32 = 50;

/// 保存页面当前状态为一个版本快照
pub fn save_version(
    conn: &Connection,
    page: &Page,
    body: &str,
    tags: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let tags_str = serde_json::to_string(tags)?;

    conn.execute(
        "INSERT INTO page_versions (id, page_id, title, body, tags, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, page.id, page.title, body, tags_str, now],
    )?;

    // 清理超出限制的旧版本
    cleanup_old_versions(conn, &page.id)?;

    Ok(())
}

fn cleanup_old_versions(
    conn: &Connection,
    page_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM page_versions WHERE page_id = ?1",
        [page_id],
        |row| row.get(0),
    )?;

    if count > MAX_VERSIONS_PER_PAGE {
        let excess = count - MAX_VERSIONS_PER_PAGE;
        conn.execute(
            "DELETE FROM page_versions WHERE rowid IN (SELECT rowid FROM page_versions WHERE page_id = ?1 ORDER BY created_at ASC LIMIT ?2)",
            params![page_id, excess],
        )?;
    }

    Ok(())
}

/// 列出页面的所有版本
pub fn list_versions(
    conn: &Connection,
    page_id: &str,
) -> Result<Vec<PageVersion>, Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare(
        "SELECT id, page_id, title, body, tags, created_at FROM page_versions WHERE page_id = ?1 ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([page_id], |row| {
        let tags_str: String = row.get(4)?;
        Ok(PageVersion {
            id: row.get(0)?,
            page_id: row.get(1)?,
            title: row.get(2)?,
            body: row.get(3)?,
            tags: serde_json::from_str(&tags_str).unwrap_or_default(),
            created_at: row.get(5)?,
        })
    })?;
    Ok(rows.filter_map(std::result::Result::ok).collect())
}

/// 获取特定版本
pub fn get_version(
    conn: &Connection,
    version_id: &str,
) -> Result<Option<PageVersion>, Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare(
        "SELECT id, page_id, title, body, tags, created_at FROM page_versions WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map([version_id], |row| {
        let tags_str: String = row.get(4)?;
        Ok(PageVersion {
            id: row.get(0)?,
            page_id: row.get(1)?,
            title: row.get(2)?,
            body: row.get(3)?,
            tags: serde_json::from_str(&tags_str).unwrap_or_default(),
            created_at: row.get(5)?,
        })
    })?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

/// 恢复到指定版本
pub fn restore_version(
    conn: &Connection,
    version_id: &str,
    ws_root: &Path,
) -> Result<bool, Box<dyn std::error::Error>> {
    let version = match get_version(conn, version_id)? {
        Some(v) => v,
        None => return Ok(false),
    };

    let page = super::page::find(conn, &version.page_id)?;
    let page = match page {
        Some(p) => p,
        None => return Ok(false),
    };

    // 先保存当前状态为版本
    let current_content = super::page::read_content(conn, &page.id, ws_root)?;
    if let Some(content) = current_content {
        save_version(conn, &page, &content.body, &content.tags)?;
    }

    // 恢复版本内容
    super::page::update_content(conn, &page.id, &version.body, ws_root)?;
    super::page::update_tags(conn, &page.id, &version.tags, ws_root)?;

    Ok(true)
}
