// MCP Tool 处理器 — initialize / tools/list / tools/call

use std::sync::Arc;

use super::{server_capabilities, server_info, JsonRpcRequest, JsonRpcResponse};
use crate::AppState;

/// MCP Tool 定义
struct ToolDef {
    name: &'static str,
    description: &'static str,
    input_schema: serde_json::Value,
}

fn get_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "page/list",
            description: "列出工作区下的页面树结构",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "workspace_id": { "type": "string", "description": "工作区 ID" }
                },
                "required": ["workspace_id"]
            }),
        },
        ToolDef {
            name: "page/read",
            description: "读取页面内容（Markdown + frontmatter 元数据）",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "page_id": { "type": "string", "description": "页面 ID" }
                },
                "required": ["page_id"]
            }),
        },
        ToolDef {
            name: "page/create",
            description: "在工作区中创建新页面",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "workspace_id": { "type": "string", "description": "工作区 ID" },
                    "title": { "type": "string", "description": "页面标题" },
                    "parent_id": { "type": "string", "description": "父页面 ID（可选）" },
                    "body": { "type": "string", "description": "页面内容（Markdown，可选）" }
                },
                "required": ["workspace_id", "title"]
            }),
        },
        ToolDef {
            name: "page/update",
            description: "更新页面内容",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "page_id": { "type": "string", "description": "页面 ID" },
                    "body": { "type": "string", "description": "新的页面内容（Markdown）" }
                },
                "required": ["page_id", "body"]
            }),
        },
        ToolDef {
            name: "page/delete",
            description: "删除页面（级联删除子页面）",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "page_id": { "type": "string", "description": "页面 ID" }
                },
                "required": ["page_id"]
            }),
        },
        ToolDef {
            name: "search",
            description: "全文搜索知识库内容",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "搜索关键词" },
                    "mode": { "type": "string", "enum": ["and", "or"], "description": "搜索模式" },
                    "limit": { "type": "integer", "description": "结果数量限制" }
                },
                "required": ["query"]
            }),
        },
    ]
}

/// 处理 initialize 请求
#[must_use] 
pub fn handle_initialize(req: &JsonRpcRequest) -> JsonRpcResponse {
    JsonRpcResponse::success(
        req.id.clone(),
        serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": server_capabilities(),
            "serverInfo": server_info(),
        }),
    )
}

/// 处理 tools/list 请求
#[must_use] 
pub fn handle_tools_list(req: &JsonRpcRequest) -> JsonRpcResponse {
    let tools: Vec<serde_json::Value> = get_tools()
        .into_iter()
        .map(|t| {
            serde_json::json!({
                "name": t.name,
                "description": t.description,
                "inputSchema": t.input_schema,
            })
        })
        .collect();

    JsonRpcResponse::success(req.id.clone(), serde_json::json!({ "tools": tools }))
}

/// 处理 tools/call 请求
pub async fn handle_tools_call(
    req: &JsonRpcRequest,
    state: &Arc<AppState>,
) -> JsonRpcResponse {
    let params = &req.params;
    let Some(tool_name) = params.get("name").and_then(|v| v.as_str()) else {
        return JsonRpcResponse::error(req.id.clone(), -32602, "Missing tool name");
    };

    let arguments = params.get("arguments").cloned().unwrap_or_else(|| serde_json::json!({}));

    let result = match tool_name {
        "page/list" => handle_page_list(state, &arguments).await,
        "page/read" => handle_page_read(state, &arguments).await,
        "page/create" => handle_page_create(state, &arguments).await,
        "page/update" => handle_page_update(state, &arguments).await,
        "page/delete" => handle_page_delete(state, &arguments).await,
        "search" => handle_search(state, &arguments).await,
        _ => Err(format!("Unknown tool: {tool_name}")),
    };

    match result {
        Ok(data) => JsonRpcResponse::success(
            req.id.clone(),
            serde_json::json!({
                "content": [{ "type": "text", "text": data.to_string() }]
            }),
        ),
        Err(e) => JsonRpcResponse::error(req.id.clone(), -32000, &e),
    }
}

async fn handle_page_list(
    state: &Arc<AppState>,
    args: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let workspace_id = args["workspace_id"]
        .as_str()
        .ok_or("Missing workspace_id")?;

    let tree = {
        let conn = state.db.lock().await;
        crate::engine::page_tree::get_tree(&conn, workspace_id, None)
            .map_err(|e| e.to_string())?
    };

    Ok(serde_json::json!({ "tree": tree }))
}

async fn handle_page_read(
    state: &Arc<AppState>,
    args: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let page_id = args["page_id"].as_str().ok_or("Missing page_id")?;

    let content = {
        let conn = state.db.lock().await;
        crate::engine::page::read_content(&conn, page_id, &state.config.storage_root)
            .map_err(|e| e.to_string())?
            .ok_or("Page not found")?
    };

    Ok(serde_json::json!({
        "title": content.title,
        "tags": content.tags,
        "created": content.created,
        "updated": content.updated,
        "body": content.body,
    }))
}

async fn handle_search(
    state: &Arc<AppState>,
    args: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let query = args["query"].as_str().ok_or("Missing query")?;
    let mode = args["mode"].as_str().unwrap_or("and");
    let limit = i32::try_from(args["limit"].as_i64().unwrap_or(20)).unwrap_or(20);

    let output = {
        let conn = state.db.lock().await;
        crate::search::search(&conn, query, mode, limit, 0)
            .map_err(|e| e.to_string())?
    };

    Ok(serde_json::json!({
        "total": output.total,
        "results": output.results,
    }))
}

async fn handle_page_create(
    state: &Arc<AppState>,
    args: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let workspace_id = args["workspace_id"].as_str().ok_or("Missing workspace_id")?;
    let title = args["title"].as_str().ok_or("Missing title")?;
    let parent_id = args["parent_id"].as_str();
    let body = args["body"].as_str().unwrap_or("");

    let page = {
        let conn = state.db.lock().await;
        crate::engine::page::create(&conn, workspace_id, parent_id, title, &state.config.storage_root)
            .map_err(|e| e.to_string())?
    };

    if !body.is_empty() {
        let conn = state.db.lock().await;
        crate::engine::page::update_content(&conn, &page.id, body, &state.config.storage_root)
            .map_err(|e| e.to_string())?;
        let _ = crate::search::index_page(&conn, &page.id, title, body, "");
    }

    Ok(serde_json::json!({
        "id": page.id,
        "title": page.title,
        "workspace_id": page.workspace_id,
        "parent_id": page.parent_id,
        "created_at": page.created_at,
    }))
}

async fn handle_page_update(
    state: &Arc<AppState>,
    args: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let page_id = args["page_id"].as_str().ok_or("Missing page_id")?;
    let body = args["body"].as_str().ok_or("Missing body")?;

    {
        let conn = state.db.lock().await;
        crate::engine::page::update_content(&conn, page_id, body, &state.config.storage_root)
            .map_err(|e| e.to_string())?;
        if let Ok(Some(content)) = crate::engine::page::read_content(&conn, page_id, &state.config.storage_root) {
            let _ = crate::search::index_page(&conn, page_id, &content.title, &content.body, &content.tags.join(", "));
        }
    }

    Ok(serde_json::json!({ "updated": true }))
}

async fn handle_page_delete(
    state: &Arc<AppState>,
    args: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let page_id = args["page_id"].as_str().ok_or("Missing page_id")?;

    let removed = {
        let conn = state.db.lock().await;
        let _ = crate::search::deindex_page(&conn, page_id);
        crate::engine::page::delete(&conn, page_id, &state.config.storage_root)
            .map_err(|e| e.to_string())?
    };

    Ok(serde_json::json!({ "removed": removed }))
}
