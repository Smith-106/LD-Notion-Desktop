// TipTap 块编辑器 — 支持文本/标题/代码/列表

import { useEditor, EditorContent } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import CodeBlockLowlight from "@tiptap/extension-code-block-lowlight";
import Placeholder from "@tiptap/extension-placeholder";
import { Table } from "@tiptap/extension-table";
import { TableRow } from "@tiptap/extension-table-row";
import { TableCell } from "@tiptap/extension-table-cell";
import { TableHeader } from "@tiptap/extension-table-header";
import Image from "@tiptap/extension-image";
import { common, createLowlight } from "lowlight";
import { Markdown as tiptapMarkdown } from "tiptap-markdown";
import { useEffect, useCallback, useRef, useState } from "react";
import { useAppStore } from "../store/appStore";
import { updatePageContent, uploadImage, exportPageHtml } from "../services/api";
import TagBar from "./TagBar";
import VersionPanel from "./VersionPanel";
import OutlinePanel from "./OutlinePanel";
import { SlashCommands } from "./SlashCommand";
import "./BlockEditor.css";

const lowlight = createLowlight(common);

const SAVE_DEBOUNCE_MS = 1200;

export default function BlockEditor() {
  const currentPage = useAppStore((s) => s.currentPage);
  const setCurrentPageContent = useAppStore((s) => s.setCurrentPageContent);
  const setCurrentPageSaved = useAppStore((s) => s.setCurrentPageSaved);
  const focusMode = useAppStore((s) => s.focusMode);
  const setFocusMode = useAppStore((s) => s.setFocusMode);
  const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const currentPageIdRef = useRef<string | null>(null);

  const [copyActive, setCopyActive] = useState(false);

  // 保持 ref 与 store 同步，避免 onUpdate 闭包捕获过时值
  useEffect(() => {
    currentPageIdRef.current = currentPage?.id ?? null;
  }, [currentPage?.id]);

  const editor = useEditor({
    extensions: [
      StarterKit.configure({
        codeBlock: false,
        heading: { levels: [1, 2, 3] },
      }),
      CodeBlockLowlight.configure({ lowlight }),
      Placeholder.configure({
        placeholder: "开始输入内容，或使用 / 命令…",
      }),
      tiptapMarkdown,
      Table.configure({ resizable: false }),
      TableRow,
      TableCell,
      TableHeader,
      Image.configure({ inline: false, allowBase64: false }),
      SlashCommands,
    ],
    editorProps: {
      attributes: {
        class: "block-editor-content",
      },
      handlePaste: (_view, event) => {
        const items = event.clipboardData?.items;
        if (!items) return false;
        for (const item of items) {
          if (item.type.startsWith("image/")) {
            const file = item.getAsFile();
            if (!file) continue;
            event.preventDefault();
            const reader = new FileReader();
            reader.onload = async () => {
              const buf = reader.result as ArrayBuffer;
              try {
                const path = await uploadImage(file.name || "paste.png", buf);
                editor?.chain().focus().setImage({ src: `/api/images/${path}` }).run();
              } catch {}
            };
            reader.readAsArrayBuffer(file);
            return true;
          }
        }
        return false;
      },
    },
    onUpdate: ({ editor }) => {
      if (saveTimer.current) clearTimeout(saveTimer.current);
      saveTimer.current = setTimeout(() => {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const md = (editor.storage as any).markdown.getMarkdown();
        setCurrentPageContent(md);
        const pageId = currentPageIdRef.current;
        if (pageId) {
          updatePageContent(pageId, md).then(
            () => setCurrentPageSaved(true),
            () => {},
          );
        }
      }, SAVE_DEBOUNCE_MS);
    },
  });

  // 代码块复制按钮 — 事件委托
  useEffect(() => {
    if (!editor) return;
    const el = document.querySelector(".block-editor-content")?.parentElement;
    if (!el) return;

    const handleClick = (e: Event) => {
      const target = (e.target as HTMLElement).closest("[data-code-copy]");
      if (!target) return;
      const pre = target.closest("pre");
      const code = pre?.querySelector("code");
      if (!code) return;
      navigator.clipboard.writeText(code.textContent || "").then(() => {
        setCopyActive(true);
        setTimeout(() => setCopyActive(false), 1500);
      });
    };

    const handleEnter = (e: Event) => {
      const pre = (e.target as HTMLElement).closest("pre");
      if (!pre || pre.querySelector("[data-code-copy]")) return;
      const btn = document.createElement("button");
      btn.setAttribute("data-code-copy", "");
      btn.textContent = "复制";
      btn.className = "code-copy-btn";
      pre.style.position = "relative";
      pre.appendChild(btn);
    };

    const handleLeave = (e: Event) => {
      const pre = (e.target as HTMLElement).closest("pre");
      pre?.querySelector("[data-code-copy]")?.remove();
    };

    el.addEventListener("click", handleClick);
    el.addEventListener("mouseenter", handleEnter, true);
    el.addEventListener("mouseleave", handleLeave, true);
    return () => {
      el.removeEventListener("click", handleClick);
      el.removeEventListener("mouseenter", handleEnter, true);
      el.removeEventListener("mouseleave", handleLeave, true);
    };
  }, [editor]);

  // Mermaid 图表渲染
  useEffect(() => {
    if (!editor) return;
    const renderMermaid = async () => {
      const container = document.querySelector(".block-editor-content");
      if (!container) return;
      const codeBlocks = container.querySelectorAll("pre code.language-mermaid");
      for (const block of codeBlocks) {
        const pre = block.parentElement;
        if (!pre || pre.querySelector(".mermaid-output")) continue;
        const source = block.textContent || "";
        try {
          const mermaid = (await import("mermaid")).default;
          mermaid.initialize({ startOnLoad: false, theme: "default" });
          const { svg } = await mermaid.render(`mermaid-${Date.now()}`, source);
          const wrapper = document.createElement("div");
          wrapper.className = "mermaid-output";
          wrapper.innerHTML = svg;
          wrapper.style.padding = "16px";
          wrapper.style.overflow = "auto";
          pre.appendChild(wrapper);
        } catch {
          // mermaid 解析失败时静默跳过
        }
      }
    };
    const timer = setTimeout(renderMermaid, 500);
    return () => clearTimeout(timer);
  }, [editor, currentPage?.body]);

  useEffect(() => {
    return () => {
      if (saveTimer.current) clearTimeout(saveTimer.current);
    };
  }, []);

  useEffect(() => {
    if (editor && currentPage?.body != null) {
      const md = currentPage.body;
      if (editor.getHTML() !== md) {
        editor.commands.setContent(md || "");
      }
    }
  }, [currentPage?.id]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "s") {
        e.preventDefault();
        if (editor && currentPage) {
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          const md = (editor.storage as any).markdown.getMarkdown();
          setCurrentPageContent(md);
          updatePageContent(currentPage.id, md).then(
            () => setCurrentPageSaved(true),
            () => {},
          );
        }
      }
    },
    [editor, currentPage, setCurrentPageContent, setCurrentPageSaved],
  );

  if (!currentPage) {
    return (
      <div className="editor-page">
        <div className="editor-welcome">
          <div className="editor-welcome-icon">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
              <path d="M12 20h9" />
              <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
            </svg>
          </div>
          <h1 className="editor-welcome-title">欢迎使用 LD-Notion Hub</h1>
          <p className="editor-welcome-desc">
            从左侧选择或创建工作区，然后选择页面开始编辑。
          </p>
        </div>
      </div>
    );
  }

  const handleExport = useCallback(() => {
    if (!editor || !currentPage) return;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const md = (editor.storage as any).markdown.getMarkdown();
    const blob = new Blob([md], { type: "text/markdown" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `${currentPage.title || "page"}.md`;
    a.click();
    URL.revokeObjectURL(url);
  }, [editor, currentPage]);

  const handleImageUpload = useCallback(() => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = "image/*";
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) return;
      const buf = await file.arrayBuffer();
      try {
        const path = await uploadImage(file.name, buf);
        editor?.chain().focus().setImage({ src: `/api/images/${path}` }).run();
      } catch {}
    };
    input.click();
  }, [editor]);

  const handleExportHtml = useCallback(async () => {
    if (!currentPage) return;
    try {
      const blob = await exportPageHtml(currentPage.id);
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `${currentPage.title || "page"}.html`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (err) {
      alert(`导出失败: ${err}`);
    }
  }, [currentPage]);

  return (
    <div className="block-editor" onKeyDown={handleKeyDown}>
      <div className="block-editor-toolbar">
        <ToolbarButton editor={editor} action="toggleHeading" args={{ level: 1 }} label="H1" />
        <ToolbarButton editor={editor} action="toggleHeading" args={{ level: 2 }} label="H2" />
        <ToolbarButton editor={editor} action="toggleHeading" args={{ level: 3 }} label="H3" />
        <div className="toolbar-separator" />
        <ToolbarButton editor={editor} action="toggleBold" label="B" />
        <ToolbarButton editor={editor} action="toggleItalic" label="I" />
        <ToolbarButton editor={editor} action="toggleCode" label="&lt;/&gt;" />
        <div className="toolbar-separator" />
        <ToolbarButton editor={editor} action="toggleBulletList" label="UL" />
        <ToolbarButton editor={editor} action="toggleOrderedList" label="OL" />
        <ToolbarButton editor={editor} action="toggleCodeBlock" label="Code" />
        <div className="toolbar-separator" />
        <button className="toolbar-btn" onClick={handleImageUpload} title="插入图片">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
            <path d="M4.5 6a1.5 1.5 0 100-3 1.5 1.5 0 000 3z" />
            <path d="M1 3.5A2.5 2.5 0 013.5 1h9A2.5 2.5 0 0115 3.5v9a2.5 2.5 0 01-2.5 2.5h-9A2.5 2.5 0 011 12.5v-9zM3.5 2A1.5 1.5 0 002 3.5v6.293l2.646-2.647a.5.5 0 01.708 0L8 9.293l2.146-2.147a.5.5 0 01.708 0L14 10.293V3.5A1.5 1.5 0 0012.5 2h-9z" />
          </svg>
        </button>
        <button className="toolbar-btn" onClick={() => editor?.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run()} title="插入表格">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
            <path d="M1 3h14v10H1V3zm1 1v3h5V4H2zm6 0v3h6V4H8zM2 8v4h5V8H2zm6 0v4h6V8H8z" />
          </svg>
        </button>
        <div className="toolbar-separator" />
        <button className="toolbar-btn" onClick={handleExport} title="导出 Markdown">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
            <path d="M2 11l6 3 6-3M8 2v10" stroke="currentColor" strokeWidth="1.5" fill="none" strokeLinecap="round" strokeLinejoin="round" />
          </svg>
        </button>
        <button className="toolbar-btn" onClick={handleExportHtml} title="导出 HTML">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
            <path d="M5 3L1 8l4 5M11 3l4 5-4 5" stroke="currentColor" strokeWidth="1.5" fill="none" strokeLinecap="round" strokeLinejoin="round" />
          </svg>
        </button>
        <OutlinePanel />
        <VersionPanel />
      </div>
      <TagBar />
      <EditorContent editor={editor} />
      <div className="editor-status-bar">
        <span className="editor-status-info">
          {currentPage && currentPage.body.trim() && (
            <>
              {currentPage.body.trim().split(/\s+/).filter(Boolean).length} 字
            </>
          )}
        </span>
        <span className="editor-status-save">
          {focusMode ? (
            <button className="focus-mode-exit" onClick={() => setFocusMode(false)}>
              退出专注模式 (F11)
            </button>
          ) : copyActive ? "已复制" : currentPage && (currentPage.saved ? "已保存" : "保存中…")}
        </span>
      </div>
    </div>
  );
}

function ToolbarButton({
  editor,
  action,
  args,
  label,
}: {
  editor: ReturnType<typeof useEditor>;
  action: string;
  args?: Record<string, unknown>;
  label: string;
}) {
  if (!editor) return null;

  const isActive =
    args && args.level
      ? editor.isActive("heading", args)
      : editor.isActive(action.replace("toggle", "").toLowerCase());

  return (
    <button
      className={`toolbar-btn ${isActive ? "toolbar-btn-active" : ""}`}
      onMouseDown={(e) => {
        e.preventDefault();
        const cmd = (editor.chain().focus() as any)[action](args);
        cmd.run();
      }}
      title={label}
    >
      {label}
    </button>
  );
}
