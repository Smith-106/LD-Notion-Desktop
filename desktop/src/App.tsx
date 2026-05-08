import { Routes, Route } from "react-router-dom";
import { useEffect, useCallback } from "react";
import Sidebar from "./components/Sidebar";
import EditorPage from "./pages/EditorPage";
import SettingsPage from "./pages/SettingsPage";
import ThemeToggle from "./components/ThemeToggle";
import KeyboardShortcuts from "./components/KeyboardShortcuts";
import { useTheme } from "./hooks/useTheme";
import { useAppStore } from "./store/appStore";
import { createPage, getPageTree } from "./services/api";
import "./App.css";

function App() {
  useTheme();
  const activeWorkspaceId = useAppStore((s) => s.activeWorkspaceId);
  const setPageTree = useAppStore((s) => s.setPageTree);
  const focusMode = useAppStore((s) => s.focusMode);
  const setFocusMode = useAppStore((s) => s.setFocusMode);

  const handleGlobalShortcut = useCallback((e: KeyboardEvent) => {
    if (e.ctrlKey && e.shiftKey && e.key === "N") {
      e.preventDefault();
      if (!activeWorkspaceId) return;
      const title = prompt("页面标题:");
      if (!title?.trim()) return;
      createPage(activeWorkspaceId, title.trim()).then(async () => {
        const tree = await getPageTree(activeWorkspaceId);
        setPageTree(tree);
      }).catch((err) => alert(`创建失败: ${err}`));
    }
    if (e.ctrlKey && e.shiftKey && e.key === "F") {
      e.preventDefault();
      document.querySelector<HTMLElement>(".search-input")?.focus();
    }
    if (e.key === "F11") {
      e.preventDefault();
      setFocusMode(!focusMode);
    }
  }, [activeWorkspaceId, setPageTree, focusMode, setFocusMode]);

  useEffect(() => {
    document.addEventListener("keydown", handleGlobalShortcut);
    return () => document.removeEventListener("keydown", handleGlobalShortcut);
  }, [handleGlobalShortcut]);

  return (
    <div className={`app-shell ${focusMode ? "focus-mode" : ""}`}>
      {!focusMode && (
        <header className="app-header" role="banner">
          <div className="header-left">
            <span className="app-logo">LD-Notion Hub</span>
          </div>
          <div className="header-right">
            <KeyboardShortcuts />
            <ThemeToggle />
          </div>
        </header>
      )}
      <div className="app-body">
        {!focusMode && <Sidebar />}
        <main className="editor-area" role="main">
          <Routes>
            <Route path="/" element={<EditorPage />} />
            <Route path="/settings" element={<SettingsPage />} />
          </Routes>
        </main>
      </div>
    </div>
  );
}

export default App;
