import { useState } from "react";
import "./KeyboardShortcuts.css";

const SHORTCUTS = [
  { keys: "Ctrl + S", desc: "保存页面" },
  { keys: "Ctrl + Shift + N", desc: "新建页面" },
  { keys: "Ctrl + Shift + F", desc: "聚焦搜索" },
  { keys: "Ctrl + K", desc: "搜索" },
  { keys: "Ctrl + Z", desc: "撤销" },
  { keys: "Ctrl + Shift + Z", desc: "重做" },
  { keys: "Ctrl + B", desc: "加粗" },
  { keys: "Ctrl + I", desc: "斜体" },
  { keys: "Ctrl + E", desc: "行内代码" },
  { keys: "Ctrl + \\", desc: "清除格式" },
];

function KeyboardShortcuts() {
  const [open, setOpen] = useState(false);

  return (
    <div className="kbd-shortcuts">
      <button
        className="kbd-shortcuts-toggle"
        onClick={() => setOpen(!open)}
        title="快捷键"
      >
        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
          <path d="M1 4a2 2 0 012-2h10a2 2 0 012 2v6a2 2 0 01-2 2H3a2 2 0 01-2-2V4zm2-1a1 1 0 00-1 1v6a1 1 0 001 1h10a1 1 0 001-1V4a1 1 0 00-1-1H3zM5 12h6v1H5v-1z" />
        </svg>
      </button>
      {open && (
        <div className="kbd-shortcuts-panel">
          <div className="kbd-shortcuts-header">
            <span>快捷键</span>
            <button className="kbd-close-btn" onClick={() => setOpen(false)}>✕</button>
          </div>
          <div className="kbd-shortcuts-list">
            {SHORTCUTS.map((s) => (
              <div key={s.keys} className="kbd-shortcut-item">
                <span className="kbd-shortcut-desc">{s.desc}</span>
                <kbd className="kbd-shortcut-keys">{s.keys}</kbd>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

export default KeyboardShortcuts;
