// 图片上传与读取

use std::path::Path;

/// 保存图片到 storage_root/uploads/，返回相对路径
pub fn save(
    storage_root: &Path,
    filename: &str,
    data: &[u8],
) -> Result<String, Box<dyn std::error::Error>> {
    let uploads_dir = storage_root.join("uploads");
    std::fs::create_dir_all(&uploads_dir)?;

    let ext = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");
    let id = uuid::Uuid::new_v4();
    let saved_name = format!("{}.{}", id, ext);
    let full_path = uploads_dir.join(&saved_name);
    std::fs::write(&full_path, data)?;

    Ok(format!("uploads/{}", saved_name))
}

/// 读取图片文件，返回 (bytes, mime_type)
pub fn read(
    storage_root: &Path,
    relative_path: &str,
) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let full_path = storage_root.join(relative_path);

    // 安全检查：防止路径遍历
    let canonical_root = storage_root.canonicalize()?;
    let canonical_file = full_path.canonicalize()?;
    if !canonical_file.starts_with(&canonical_root) {
        return Err("invalid path".into());
    }

    let bytes = std::fs::read(&full_path)?;
    let ext = full_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");
    let mime = match ext {
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        _ => "image/png",
    };
    Ok((bytes, mime.to_string()))
}
