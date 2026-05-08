// API 客户端 — 连接 LD-Notion Hub 后端

const API_BASE = "";

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      ...init?.headers as Record<string, string>,
    },
  });
  if (!res.ok) {
    let message = `HTTP ${res.status}`;
    try {
      const body = await res.json();
      if (body.error) message = body.error;
    } catch {}
    throw new Error(message);
  }
  const data = await res.json();
  if (!data.ok && data.error) {
    throw new Error(data.error);
  }
  return data;
}

// ── 工作区 ──

export interface Workspace {
  id: string;
  name: string;
  root_path: string;
  created_at: string;
  updated_at: string;
}

export async function listWorkspaces(): Promise<Workspace[]> {
  const data = await request<{ data: Workspace[] }>("/api/workspaces");
  return data.data;
}

export async function createWorkspace(name: string): Promise<Workspace> {
  const data = await request<{ data: Workspace }>("/api/workspaces", {
    method: "POST",
    body: JSON.stringify({ name }),
  });
  return data.data;
}

export async function deleteWorkspace(id: string): Promise<void> {
  await request(`/api/workspaces/${id}`, { method: "DELETE" });
}

export async function renameWorkspace(id: string, name: string): Promise<Workspace> {
  const data = await request<{ data: Workspace }>(`/api/workspaces/${id}`, {
    method: "PUT",
    body: JSON.stringify({ name }),
  });
  return data.data;
}

// ── 页面 ──

export interface Page {
  id: string;
  workspace_id: string;
  parent_id: string | null;
  title: string;
  slug: string;
  file_path: string;
  sort_order: number;
  is_folder: boolean;
  is_pinned: boolean;
  created_at: string;
  updated_at: string;
}

export interface PageContent {
  title: string;
  tags: string[];
  created: string;
  updated: string;
  body: string;
}

export interface PageTreeNode {
  id: string;
  workspace_id: string;
  parent_id: string | null;
  title: string;
  slug: string;
  file_path: string;
  sort_order: number;
  is_folder: boolean;
  created_at: string;
  updated_at: string;
  children: PageTreeNode[];
}

export async function createPage(
  workspaceId: string,
  title: string,
  parentId?: string,
  isFolder?: boolean,
): Promise<Page> {
  const reqBody: Record<string, unknown> = { workspace_id: workspaceId, title };
  if (parentId) reqBody.parent_id = parentId;
  if (isFolder) reqBody.is_folder = true;
  const data = await request<{ data: Page }>("/api/pages", {
    method: "POST",
    body: JSON.stringify(reqBody),
  });
  return data.data;
}

export async function getPage(id: string): Promise<Page> {
  const data = await request<{ data: Page }>(`/api/pages/${id}`);
  return data.data;
}

export async function getPageContent(id: string): Promise<PageContent> {
  const data = await request<{ data: PageContent }>(`/api/pages/${id}/content`);
  return data.data;
}

export async function updatePageContent(
  id: string,
  body: string,
): Promise<void> {
  await request(`/api/pages/${id}/content`, {
    method: "PUT",
    body: JSON.stringify({ body }),
  });
}

export async function deletePage(id: string): Promise<void> {
  await request(`/api/pages/${id}`, { method: "DELETE" });
}

export async function renamePage(id: string, title: string): Promise<Page> {
  const data = await request<{ data: Page }>(`/api/pages/${id}/rename`, {
    method: "PUT",
    body: JSON.stringify({ title }),
  });
  return data.data;
}

export async function movePage(id: string, parentId: string | null): Promise<Page> {
  const data = await request<{ data: Page }>(`/api/pages/${id}/move`, {
    method: "PUT",
    body: JSON.stringify({ parent_id: parentId }),
  });
  return data.data;
}

export async function importPage(
  workspaceId: string,
  title: string,
  body: string,
  parentId?: string,
): Promise<Page> {
  const reqBody: Record<string, string> = { workspace_id: workspaceId, title, body };
  if (parentId) reqBody.parent_id = parentId;
  const data = await request<{ data: Page }>("/api/pages/import", {
    method: "POST",
    body: JSON.stringify(reqBody),
  });
  return data.data;
}

export async function getPageTree(
  workspaceId: string,
): Promise<PageTreeNode[]> {
  const data = await request<{ data: PageTreeNode[] }>(
    `/api/workspaces/${workspaceId}/tree`,
  );
  return data.data;
}

// ── 标签 ──

export interface TagInfo {
  name: string;
  count: number;
}

export async function listTags(workspaceId: string): Promise<TagInfo[]> {
  const data = await request<{ data: TagInfo[] }>(
    `/api/workspaces/${workspaceId}/tags`,
  );
  return data.data;
}

export async function updatePageTags(
  id: string,
  tags: string[],
): Promise<void> {
  await request(`/api/pages/${id}/tags`, {
    method: "PUT",
    body: JSON.stringify({ tags }),
  });
}

// ── 最近 & 收藏 ──

export async function listRecentPages(
  workspaceId: string,
  limit = 10,
): Promise<Page[]> {
  const data = await request<{ data: Page[] }>(
    `/api/workspaces/${workspaceId}/recent?limit=${limit}`,
  );
  return data.data;
}

export async function listPinnedPages(
  workspaceId: string,
): Promise<Page[]> {
  const data = await request<{ data: Page[] }>(
    `/api/workspaces/${workspaceId}/pinned`,
  );
  return data.data;
}

export async function togglePagePin(id: string): Promise<Page> {
  const data = await request<{ data: Page }>(`/api/pages/${id}/pin`, {
    method: "PUT",
  });
  return data.data;
}

export async function duplicatePage(id: string): Promise<Page> {
  const data = await request<{ data: Page }>(`/api/pages/${id}/duplicate`, {
    method: "POST",
  });
  return data.data;
}

export async function getWorkspaceStats(
  workspaceId: string,
): Promise<{ page_count: number }> {
  const data = await request<{ data: { page_count: number } }>(
    `/api/workspaces/${workspaceId}/stats`,
  );
  return data.data;
}

// ── 搜索 ──

export interface SearchResult {
  page_id: string;
  title: string;
  snippet: string;
  score: number;
}

export async function searchPages(
  query: string,
  mode: "and" | "or" = "and",
): Promise<SearchResult[]> {
  const data = await request<{ data: { results: SearchResult[] } }>(
    `/api/search?q=${encodeURIComponent(query)}&mode=${mode}`,
  );
  return data.data.results;
}

// ── 回收站 ──

export interface TrashItem {
  id: string;
  workspace_id: string;
  parent_id: string | null;
  title: string;
  is_folder: boolean;
  original_created_at: string;
  original_updated_at: string;
  deleted_at: string;
}

export async function listTrash(workspaceId: string): Promise<TrashItem[]> {
  const data = await request<{ data: TrashItem[] }>(
    `/api/workspaces/${workspaceId}/trash`,
  );
  return data.data;
}

export async function restorePage(id: string): Promise<void> {
  await request(`/api/pages/${id}/restore`, { method: "POST" });
}

export async function purgePage(id: string): Promise<void> {
  await request(`/api/pages/${id}/purge`, { method: "DELETE" });
}

export async function emptyTrash(workspaceId: string): Promise<void> {
  await request(`/api/workspaces/${workspaceId}/trash`, { method: "DELETE" });
}

// ── 排序 ──

export async function reorderPages(pageIds: string[]): Promise<void> {
  await request("/api/pages/reorder", {
    method: "PUT",
    body: JSON.stringify({ page_ids: pageIds }),
  });
}

// ── 图片 ──

export async function uploadImage(filename: string, data: ArrayBuffer): Promise<string> {
  const res = await fetch(`/api/images/upload?filename=${encodeURIComponent(filename)}`, {
    method: "POST",
    body: data,
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  const json = await res.json();
  if (!json.ok) throw new Error(json.error);
  return json.path;
}

// ── 备份 ──

export interface PageVersion {
  id: string;
  page_id: string;
  title: string;
  body: string;
  tags: string[];
  created_at: string;
}

export async function listPageVersions(pageId: string): Promise<PageVersion[]> {
  const data = await request<{ data: PageVersion[] }>(`/api/pages/${pageId}/versions`);
  return data.data;
}

export async function restorePageVersion(versionId: string): Promise<void> {
  await request(`/api/pages/${versionId}/versions/restore`, { method: "POST" });
}

export async function exportPageHtml(pageId: string): Promise<Blob> {
  const res = await fetch(`/api/pages/${pageId}/html`);
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.blob();
}

export async function exportWorkspace(workspaceId: string): Promise<Blob> {
  const res = await fetch(`/api/workspaces/${workspaceId}/export`);
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.blob();
}

export async function importWorkspace(workspaceName: string, data: ArrayBuffer): Promise<string> {
  const res = await fetch(`/api/workspaces/import?workspace_name=${encodeURIComponent(workspaceName)}`, {
    method: "POST",
    body: data,
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  const json = await res.json();
  if (!json.ok) throw new Error(json.error);
  return json.workspace_id;
}
