import { useCallback, useEffect, useState } from "react";
import { useAppStore } from "../store/appStore";
import "./OutlinePanel.css";

interface HeadingItem {
  level: number;
  text: string;
  id: string;
}

function extractHeadings(container: HTMLElement | null): HeadingItem[] {
  if (!container) return [];
  const headings: HeadingItem[] = [];
  container.querySelectorAll("h1, h2, h3").forEach((el, i) => {
    const id = `heading-${i}`;
    el.id = id;
    headings.push({
      level: parseInt(el.tagName[1]),
      text: el.textContent || "",
      id,
    });
  });
  return headings;
}

function OutlinePanel() {
  const currentPage = useAppStore((s) => s.currentPage);
  const [headings, setHeadings] = useState<HeadingItem[]>([]);
  const [open, setOpen] = useState(false);

  useEffect(() => {
    if (!open) return;
    const container = document.querySelector(".block-editor-content");
    setHeadings(extractHeadings(container as HTMLElement));
  }, [open, currentPage?.body]);

  const handleJump = useCallback((id: string) => {
    document.getElementById(id)?.scrollIntoView({ behavior: "smooth", block: "start" });
  }, []);

  if (!currentPage || !open) {
    return (
      <button
        className="outline-toggle-btn"
        onClick={() => setOpen(true)}
        title="页面大纲"
      >
        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
          <path d="M1 3h10v1H1V3zm0 3h7v1H1V6zm0 3h10v1H1V9zm0 3h7v1H1v-1z" />
        </svg>
      </button>
    );
  }

  return (
    <div className="outline-panel">
      <button
        className="outline-toggle-btn"
        onClick={() => setOpen(false)}
        title="关闭大纲"
      >
        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
          <path d="M1 3h10v1H1V3zm0 3h7v1H1V6zm0 3h10v1H1V9zm0 3h7v1H1v-1z" />
        </svg>
      </button>
      {open && headings.length > 0 && (
        <div className="outline-dropdown">
          <div className="outline-header">
            <span>大纲</span>
          </div>
          <div className="outline-list">
            {headings.map((h) => (
              <button
                key={h.id}
                className={`outline-item outline-level-${h.level}`}
                onClick={() => handleJump(h.id)}
              >
                {h.text || "(无标题)"}
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

export { extractHeadings };
export default OutlinePanel;
