-- LD-Notion Hub 元数据库 schema
-- 所有建表语句均为幂等（IF NOT EXISTS），可重复执行

-- 工作区表
CREATE TABLE IF NOT EXISTS workspaces (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    root_path TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 页面表（邻接表模型，支持文件夹和普通页面）
CREATE TABLE IF NOT EXISTS pages (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    parent_id TEXT REFERENCES pages(id),
    title TEXT NOT NULL,
    slug TEXT NOT NULL,
    file_path TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_folder INTEGER NOT NULL DEFAULT 0,
    is_pinned INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(workspace_id, file_path)
);

-- 增量迁移：为旧数据库添加 is_pinned 列（幂等，忽略已存在的情况）
-- SQLite 不支持 IF NOT EXISTS for ALTER TABLE，通过忽略错误实现幂等
-- 实际在 Rust 代码中处理

-- 增量迁移：为旧数据库添加 deleted_at 列（软删除）
-- 实际在 Rust 代码中处理

-- 回收站快照表（记录删除时的页面元信息）
CREATE TABLE IF NOT EXISTS trash (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL,
    parent_id TEXT,
    title TEXT NOT NULL,
    slug TEXT NOT NULL,
    file_path TEXT NOT NULL,
    is_folder INTEGER NOT NULL DEFAULT 0,
    body TEXT NOT NULL DEFAULT '',
    tags TEXT NOT NULL DEFAULT '[]',
    original_created_at TEXT NOT NULL,
    original_updated_at TEXT NOT NULL,
    deleted_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 页面树表（物化路径缓存，加速祖先/后代查询）
CREATE TABLE IF NOT EXISTS page_tree (
    ancestor_id TEXT NOT NULL REFERENCES pages(id),
    descendant_id TEXT NOT NULL REFERENCES pages(id),
    depth INTEGER NOT NULL,
    PRIMARY KEY (ancestor_id, descendant_id)
);

-- 全文搜索索引（FTS5，支持中文 unicode61 分词）
CREATE VIRTUAL TABLE IF NOT EXISTS fts_index USING fts5(
    page_id,
    title,
    content,
    tags,
    tokenize='unicode61'
);
