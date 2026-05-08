# LD-Notion Desktop — 本地知识库桌面应用

基于 Tauri v2 + React 18 + Rust + SQLite 的独立桌面知识库应用，数据完全本地存储。

## 功能

- **工作区管理** — 多工作区创建、重命名、删除、统计
- **页面树** — 层级页面结构，拖拽移动，闭包表存储
- **Markdown 编辑器** — TipTap 块编辑器，支持标题/代码块/列表，自动保存
- **全文搜索** — FTS5 索引，支持 AND/OR 模式，Ctrl+K 快捷键
- **标签系统** — 页面标签编辑，工作区级标签过滤
- **收藏 & 最近** — 页面置顶收藏，最近编辑快速访问
- **页面操作** — 复制页面、批量删除、Markdown 导入导出
- **MCP 端点** — Model Context Protocol 支持，AI 助手可连接知识库
- **主题** — 亮色/暗色/自动切换

## 技术栈

| 层 | 技术 |
|----|------|
| 后端 | Rust + Axum + SQLite (FTS5) |
| 前端 | Tauri v2 + React 18 + TypeScript |
| 编辑器 | TipTap + tiptap-markdown |
| 状态管理 | Zustand |
| 存储 | SQLite 数据库 + Markdown 文件 (frontmatter) |

## 开发

```bash
# 后端
cd backend
cargo run

# 前端
cd desktop
npm install
npm run tauri dev
```

## 版本历史

| 版本 | 里程碑 | 主要功能 |
|------|--------|----------|
| v0.9.0 | M8 | 页面复制、代码块复制、工作区统计、批量删除 |
| v0.8.0 | M7 | 工作区重命名、编辑器状态栏、Ctrl+K |
| v0.7.0 | M6 | 最近编辑、页面收藏 |
| v0.6.0 | M5 | 标签系统 |
| v0.5.1 | M4 | 页面管理、工作区、导入导出 |
| v0.4.2 | M3 | 全文搜索 |
| v0.4.0 | M2 | 前端集成 |
| v0.3.2 | M1 | MVP 后端 |

## 许可证

MIT License
