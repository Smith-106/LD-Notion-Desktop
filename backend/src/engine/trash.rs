// 回收站 — 软删除与恢复

use rusqlite::{params, Connection};
use std::path::Path;

use super::{TrashItem, Page, MarkdownContent};
use super::markdown_io;

/// 将页面移入回收站（软删除，级联处理子页面）
pub fn soft_delete(
    conn: &Connection,
    id: &str,
    ws_root: &Path,
) -> Result<bool, Box<dyn std::error::Error>> {
    let Some(page) = super::page::find(conn, id)? else {
        return Ok(false);
    };
    let now = chrono::Utc::now().to_rfc3339();

    // 收集所有后代
    let descendants: Vec<String> = {
        let mut stmt = conn.prepare(
            "SELECT descendant_id FROM page_tree WHERE ancestor_id = ?1 AND depth > 0",
        )?;
        let rows = stmt.query_map([id], |row| row.get::<_, String>(0))?;
        rows.filter_map(std::result::Result::ok).collect()
    };

    // 将自身和后代移入 trash 表
    move_to_trash(conn, &page, ws_root, &now)?;
    for desc_id in &descendants {
        if let Some(child) = super::page::find(conn, desc_id)? {
            move_to_trash(conn, &child, ws_root, &now)?;
        }
    }

    // 清除后代的 parent_id
    for desc_id in &descendants {
        conn.execute("UPDATE pages SET parent_id = NULL WHERE id = ?1", [desc_id])?;
    }
    // 删除 page_tree 关系
    for desc_id in &descendants {
        conn.execute("DELETE FROM page_tree WHERE descendant_id = ?1", [desc_id])?;
    }
    conn.execute("DELETE FROM page_tree WHERE ancestor_id = ?1 OR descendant_id = ?1", [id])?;

    // 软删除：标记 deleted_at
    conn.execute("UPDATE pages SET deleted_at = ?1 WHERE id = ?2", params![&now, id])?;
    for desc_id in &descendants {
        conn.execute("UPDATE pages SET deleted_at = ?1 WHERE id = ?2", params![&now, desc_id])?;
    }

    // 删除搜索索引
    let _ = crate::search::deindex_page(conn, id);
    for desc_id in &descendants {
        let _ = crate::search::deindex_page(conn, desc_id);
    }

    Ok(true)
}

fn move_to_trash(
    conn: &Connection,
    page: &Page,
    ws_root: &Path,
    deleted_at: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (body, tags) = if page.is_folder {
        (String::new(), "[]".to_string())
    } else {
        let full_path = ws_root.join(&page.file_path);
        match markdown_io::read(&full_path) {
            Ok(content) => (content.body, serde_json::to_string(&content.tags)?),
            Err(_) => (String::new(), "[]".to_string()),
        }
    };

    conn.execute(
        "INSERT OR IGNORE INTO trash (id, workspace_id, parent_id, title, slug, file_path, is_folder, body, tags, original_created_at, original_updated_at, deleted_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            page.id, page.workspace_id, page.parent_id, page.title,
            page.slug, page.file_path, page.is_folder, body, tags,
            page.created_at, page.updated_at, deleted_at
        ],
    )?;
    Ok(())
}

/// 列出回收站
pub fn list(conn: &Connection, workspace_id: &str) -> Result<Vec<TrashItem>, Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare(
        "SELECT id, workspace_id, parent_id, title, is_folder, original_created_at, original_updated_at, deleted_at
         FROM trash WHERE workspace_id = ?1 ORDER BY deleted_at DESC",
    )?;
    let rows = stmt.query_map([workspace_id], |row| {
        Ok(TrashItem {
            id: row.get(0)?,
            workspace_id: row.get(1)?,
            parent_id: row.get(2)?,
            title: row.get(3)?,
            is_folder: row.get(4)?,
            original_created_at: row.get(5)?,
            original_updated_at: row.get(6)?,
            deleted_at: row.get(7)?,
        })
    })?;
    Ok(rows.filter_map(std::result::Result::ok).collect())
}

/// 从回收站恢复页面
pub fn restore(
    conn: &Connection,
    id: &str,
    ws_root: &Path,
) -> Result<bool, Box<dyn std::error::Error>> {
    let _item: TrashItem = {
        let mut stmt = conn.prepare(
            "SELECT id, workspace_id, parent_id, title, is_folder, original_created_at, original_updated_at, deleted_at
             FROM trash WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map([id], |row| {
            Ok(TrashItem {
                id: row.get(0)?,
                workspace_id: row.get(1)?,
                parent_id: row.get(2)?,
                title: row.get(3)?,
                is_folder: row.get(4)?,
                original_created_at: row.get(5)?,
                original_updated_at: row.get(6)?,
                deleted_at: row.get(7)?,
            })
        })?;
        match rows.next() {
            Some(row) => row?,
            None => return Ok(false),
        }
    };

    // 清除 deleted_at 标记
    conn.execute("UPDATE pages SET deleted_at = NULL WHERE id = ?1", [id])?;

    // 恢复 Markdown 文件内容
    let body: String = conn.query_row(
        "SELECT body FROM trash WHERE id = ?1", [id], |row| row.get(0),
    )?;
    let tags_str: String = conn.query_row(
        "SELECT tags FROM trash WHERE id = ?1", [id], |row| row.get(0),
    )?;
    let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();

    let page = super::page::find(conn, id)?;
    if let Some(page) = &page {
        let full_path = ws_root.join(&page.file_path);
        let now = chrono::Utc::now().to_rfc3339();
        let md = MarkdownContent {
            title: page.title.clone(),
            tags,
            created: page.created_at.clone(),
            updated: now,
            body,
        };
        markdown_io::write(&full_path, &md)?;

        // 重建 page_tree
        conn.execute(
            "INSERT OR IGNORE INTO page_tree (ancestor_id, descendant_id, depth) VALUES (?1, ?1, 0)",
            [id],
        )?;

        // 重建搜索索引
        let _ = crate::search::index_page(conn, id, &page.title, &md.body, &md.tags.join(", "));
    }

    // 从 trash 表移除
    conn.execute("DELETE FROM trash WHERE id = ?1", [id])?;

    Ok(page.is_some())
}

/// 永久删除回收站中的页面
pub fn purge(
    conn: &Connection,
    id: &str,
    ws_root: &Path,
) -> Result<bool, Box<dyn std::error::Error>> {
    // 删除文件
    let page = super::page::find(conn, id)?;
    if let Some(page) = &page {
        let full_path = ws_root.join(&page.file_path);
        let _ = std::fs::remove_file(full_path);
    }

    // 从所有表中彻底删除
    conn.execute("DELETE FROM page_tree WHERE descendant_id = ?1", [id])?;
    conn.execute("DELETE FROM fts_index WHERE page_id = ?1", [id])?;
    conn.execute("DELETE FROM trash WHERE id = ?1", [id])?;
    conn.execute("DELETE FROM pages WHERE id = ?1", [id])?;
    Ok(true)
}

/// 清空回收站（永久删除所有回收站条目）
pub fn empty(conn: &Connection, workspace_id: &str, ws_root: &Path) -> Result<u64, Box<dyn std::error::Error>> {
    let items = list(conn, workspace_id)?;
    let count = items.len() as u64;
    for item in &items {
        purge(conn, &item.id, ws_root)?;
    }
    Ok(count)
}
