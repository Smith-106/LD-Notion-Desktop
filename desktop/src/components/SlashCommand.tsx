// Slash 命令 — 编辑器内输入 / 弹出命令菜单

import { Extension } from "@tiptap/core";
import Suggestion from "@tiptap/suggestion";
import { Editor, Range as TipTapRange } from "@tiptap/react";
import { useState, useEffect, useRef } from "react";
import { createRoot } from "react-dom/client";
import "./SlashCommand.css";

interface CommandItem {
  title: string;
  description: string;
  icon: string;
  command: (props: { editor: Editor; range: TipTapRange }) => void;
}

const COMMANDS: CommandItem[] = [
  {
    title: "标题 1",
    description: "大标题",
    icon: "H1",
    command: ({ editor, range }) => {
      editor.chain().focus().deleteRange(range).setHeading({ level: 1 }).run();
    },
  },
  {
    title: "标题 2",
    description: "中标题",
    icon: "H2",
    command: ({ editor, range }) => {
      editor.chain().focus().deleteRange(range).setHeading({ level: 2 }).run();
    },
  },
  {
    title: "标题 3",
    description: "小标题",
    icon: "H3",
    command: ({ editor, range }) => {
      editor.chain().focus().deleteRange(range).setHeading({ level: 3 }).run();
    },
  },
  {
    title: "无序列表",
    description: "项目符号列表",
    icon: "•",
    command: ({ editor, range }) => {
      editor.chain().focus().deleteRange(range).toggleBulletList().run();
    },
  },
  {
    title: "有序列表",
    description: "编号列表",
    icon: "1.",
    command: ({ editor, range }) => {
      editor.chain().focus().deleteRange(range).toggleOrderedList().run();
    },
  },
  {
    title: "代码块",
    description: "代码块（语法高亮）",
    icon: "</>",
    command: ({ editor, range }) => {
      editor.chain().focus().deleteRange(range).toggleCodeBlock().run();
    },
  },
  {
    title: "引用",
    description: "引用块",
    icon: "❝",
    command: ({ editor, range }) => {
      editor.chain().focus().deleteRange(range).toggleBlockquote().run();
    },
  },
  {
    title: "分割线",
    description: "水平分割线",
    icon: "—",
    command: ({ editor, range }) => {
      editor.chain().focus().deleteRange(range).setHorizontalRule().run();
    },
  },
  {
    title: "表格",
    description: "3×3 表格",
    icon: "⊞",
    command: ({ editor, range }) => {
      editor.chain().focus().deleteRange(range).insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run();
    },
  },
  {
    title: "图片",
    description: "上传图片",
    icon: "🖼",
    command: ({ editor, range }) => {
      editor.chain().focus().deleteRange(range).run();
      const input = document.createElement("input");
      input.type = "file";
      input.accept = "image/*";
      input.onchange = async () => {
        const file = input.files?.[0];
        if (!file) return;
        const buf = await file.arrayBuffer();
        try {
          const { uploadImage } = await import("../services/api");
          const path = await uploadImage(file.name, buf);
          editor.chain().focus().setImage({ src: `/api/images/${path}` }).run();
        } catch {}
      };
      input.click();
    },
  },
];

function filterCommands(query: string): CommandItem[] {
  return COMMANDS.filter(
    (item) =>
      item.title.toLowerCase().includes(query.toLowerCase()) ||
      item.description.toLowerCase().includes(query.toLowerCase()),
  );
}

let activePopup: { destroy: () => void } | null = null;

function showPopup(
  items: CommandItem[],
  command: (item: CommandItem) => void,
  rect: DOMRect | null,
) {
  removePopup();
  const container = document.createElement("div");
  container.className = "slash-command-container";
  if (rect) {
    container.style.position = "fixed";
    container.style.left = `${rect.left}px`;
    container.style.top = `${rect.bottom + 4}px`;
    container.style.zIndex = "9999";
  }
  document.body.appendChild(container);

  const root = createRoot(container);
  root.render(
    <SlashCommandList
      items={items}
      command={command}
    />,
  );

  activePopup = {
    destroy: () => {
      root.unmount();
      container.remove();
    },
  };
}

function removePopup() {
  if (activePopup) {
    activePopup.destroy();
    activePopup = null;
  }
}

function SlashCommandList({
  items,
  command,
}: {
  items: CommandItem[];
  command: (item: CommandItem) => void;
}) {
  const [selectedIndex, setSelectedIndex] = useState(0);
  const listRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    setSelectedIndex(0);
  }, [items]);

  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        setSelectedIndex((i) => (i + 1) % items.length);
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setSelectedIndex((i) => (i - 1 + items.length) % items.length);
      } else if (e.key === "Enter") {
        e.preventDefault();
        if (items[selectedIndex]) command(items[selectedIndex]);
      }
    };
    document.addEventListener("keydown", handleKey, true);
    return () => document.removeEventListener("keydown", handleKey, true);
  }, [items, selectedIndex, command]);

  useEffect(() => {
    const el = listRef.current?.children[selectedIndex] as HTMLElement;
    el?.scrollIntoView({ block: "nearest" });
  }, [selectedIndex]);

  if (items.length === 0) {
    return (
      <div className="slash-command-dropdown">
        <p className="slash-command-empty">无匹配命令</p>
      </div>
    );
  }

  return (
    <div className="slash-command-dropdown" ref={listRef}>
      {items.map((item, index) => (
        <button
          key={item.title}
          className={`slash-command-item ${index === selectedIndex ? "slash-command-item-active" : ""}`}
          onClick={() => command(item)}
          onMouseEnter={() => setSelectedIndex(index)}
        >
          <span className="slash-command-icon">{item.icon}</span>
          <div className="slash-command-text">
            <span className="slash-command-title">{item.title}</span>
            <span className="slash-command-desc">{item.description}</span>
          </div>
        </button>
      ))}
    </div>
  );
}

export const SlashCommands = Extension.create({
  name: "slashCommands",

  addOptions() {
    return {
      suggestion: {
        char: "/",
        items: ({ query }: { query: string }) => filterCommands(query),
        render: () => ({
          onStart(props: any) {
            showPopup(props.items, props.command, props.clientRect?.() ?? null);
          },
          onUpdate(props: any) {
            showPopup(props.items, props.command, props.clientRect?.() ?? null);
          },
          onKeyDown(props: any) {
            if (props.event.key === "Escape") {
              removePopup();
              return true;
            }
            return false;
          },
          onExit() {
            removePopup();
          },
        }),
      },
    };
  },

  addProseMirrorPlugins() {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return [Suggestion((this as any).options.suggestion)];
  },
});
