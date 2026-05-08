// 工作区备份与恢复

use rusqlite::Connection;
use std::io::{Read as IoRead, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

/// 导出工作区为 ZIP 字节
pub fn export_workspace(
    conn: &Connection,
    workspace_id: &str,
    storage_root: &Path,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let ws = super::workspace::find(conn, workspace_id)?
        .ok_or("工作区不存在")?;

    let mut buf = std::io::Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut buf);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // 写入元数据
    let meta = serde_json::json!({
        "workspace": ws,
        "exported_at": chrono::Utc::now().to_rfc3339(),
    });
    zip.start_file("metadata.json", options)?;
    zip.write_all(serde_json::to_string_pretty(&meta)?.as_bytes())?;

    // 写入所有页面
    let pages = super::page::list_by_workspace(conn, workspace_id)?;
    for page in &pages {
        if page.is_folder {
            continue;
        }
        let full_path = storage_root.join(&page.file_path);
        if let Ok(content) = std::fs::read_to_string(&full_path) {
            zip.start_file(&format!("pages/{}.md", page.slug), options)?;
            zip.write_all(content.as_bytes())?;
        }
    }

    // 写入数据库快照
    let pages_json = serde_json::to_string_pretty(&pages)?;
    zip.start_file("pages.json", options)?;
    zip.write_all(pages_json.as_bytes())?;

    zip.finish()?;
    Ok(buf.into_inner())
}

/// 从 ZIP 字节导入工作区
pub fn import_workspace(
    conn: &Connection,
    storage_root: &Path,
    data: &[u8],
    workspace_name: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let reader = std::io::Cursor::new(data);
    let mut archive = ZipArchive::new(reader)?;

    // 创建新工作区
    let ws = super::workspace::create(conn, workspace_name, storage_root)?;

    // 读取 pages.json
    let pages_json: String = {
        let mut file = archive.by_name("pages.json")?;
        let mut s = String::new();
        file.read_to_string(&mut s)?;
        s
    };
    let pages: Vec<super::Page> = serde_json::from_str(&pages_json)?;

    // 导入页面
    for page in &pages {
        let slug = &page.slug;
        let file_name = format!("pages/{}.md", slug);
        let body = match archive.by_name(&file_name) {
            Ok(mut f) => {
                let mut s = String::new();
                f.read_to_string(&mut s)?;
                s
            }
            Err(_) => String::new(),
        };

        let new_page = super::page::create(
            conn,
            &ws.id,
            None,
            &page.title,
            storage_root,
        )?;

        if !body.is_empty() {
            let _ = super::page::update_content(conn, &new_page.id, &body, storage_root);
        }
    }

    Ok(ws.id)
}
