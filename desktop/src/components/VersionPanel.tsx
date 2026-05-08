import { useEffect, useState, useCallback } from "react";
import { useAppStore } from "../store/appStore";
import { listPageVersions, restorePageVersion, getPageContent } from "../services/api";
import type { PageVersion } from "../services/api";
import "./VersionPanel.css";

function VersionPanel() {
  const currentPage = useAppStore((s) => s.currentPage);
  const setCurrentPage = useAppStore((s) => s.setCurrentPage);
  const setCurrentTags = useAppStore((s) => s.setCurrentTags);
  const [versions, setVersions] = useState<PageVersion[]>([]);
  const [open, setOpen] = useState(false);

  useEffect(() => {
    if (open && currentPage?.id) {
      listPageVersions(currentPage.id).then(setVersions).catch(() => setVersions([]));
    }
  }, [open, currentPage?.id]);

  const handleRestore = useCallback(async (version: PageVersion) => {
    if (!confirm(`恢复到 ${new Date(version.created_at).toLocaleString()} 的版本？当前内容会保存为新版本。`)) return;
    try {
      await restorePageVersion(version.id);
      // 重新加载当前页面内容
      if (currentPage?.id) {
        const content = await getPageContent(currentPage.id);
        setCurrentPage({ id: currentPage.id, title: content.title, body: content.body, saved: true });
        setCurrentTags(content.tags);
      }
      // 刷新版本列表
      if (currentPage?.id) {
        listPageVersions(currentPage.id).then(setVersions).catch(() => {});
      }
    } catch (err) {
      alert(`恢复失败: ${err}`);
    }
  }, [currentPage, setCurrentPage, setCurrentTags]);

  if (!currentPage) return null;

  return (
    <div className="version-panel">
      <button
        className="version-toggle-btn"
        onClick={() => setOpen(!open)}
        title="版本历史"
      >
        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8 1a7 7 0 100 14A7 7 0 008 1zm0 1a6 6 0 014.243 1.757l-.708.708A5 5 0 008 3 5 5 0 003 8h1.5L2 10.5 0 8h1.5a6.5 6.5 0 0111.743-3.757l-.708.708A6 6 0 008 2z" fillRule="evenodd" />
          <circle cx="8" cy="8" r="2" />
        </svg>
      </button>
      {open && (
        <div className="version-dropdown">
          <div className="version-dropdown-header">
            <span>版本历史</span>
            <button className="version-close-btn" onClick={() => setOpen(false)}>✕</button>
          </div>
          {versions.length === 0 ? (
            <p className="version-empty">暂无历史版本</p>
          ) : (
            <div className="version-list">
              {versions.map((v) => (
                <div key={v.id} className="version-item">
                  <div className="version-info">
                    <span className="version-time">{new Date(v.created_at).toLocaleString()}</span>
                    <span className="version-preview">{v.body.slice(0, 60) || "(空)"}</span>
                  </div>
                  <button
                    className="version-restore-btn"
                    onClick={() => handleRestore(v)}
                  >
                    恢复
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

export default VersionPanel;
