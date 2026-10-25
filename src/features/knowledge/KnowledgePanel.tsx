import { useEffect, useState } from "react";

import * as bridge from "../../lib/bridge";
import type { KnowledgeEntry, KnowledgeKind } from "../../lib/types";

const KINDS: { id: KnowledgeKind; label: string }[] = [
  { id: "brushes", label: "画笔" },
  { id: "palettes", label: "配色" },
  { id: "shapes", label: "形状" },
  { id: "design", label: "规范" },
];

/** 知识库面板：四分区浏览/编辑 + 注册为 DSH 工作区。 */
export function KnowledgePanel() {
  const [kind, setKind] = useState<KnowledgeKind>("palettes");
  const [entries, setEntries] = useState<KnowledgeEntry[]>([]);
  const [name, setName] = useState("");
  const [content, setContent] = useState("");
  const [status, setStatus] = useState<string | null>(null);
  const [root, setRoot] = useState<string>("");

  const refresh = async (nextKind: KnowledgeKind = kind) => {
    try {
      setEntries(await bridge.listKnowledge(nextKind));
    } catch (error) {
      setStatus(error instanceof Error ? error.message : String(error));
    }
  };

  useEffect(() => {
    void bridge.knowledgeDirs().then((dirs) => setRoot(dirs.root)).catch(() => undefined);
    void refresh(kind);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [kind]);

  const load = async (entry: KnowledgeEntry) => {
    try {
      const full = await bridge.readKnowledge(kind, entry.name);
      setName(full.name);
      setContent(full.content ?? "");
    } catch (error) {
      setStatus(error instanceof Error ? error.message : String(error));
    }
  };

  const save = async () => {
    try {
      await bridge.saveKnowledgeEntry(kind, name, content);
      setStatus("已保存，并刷新 AGENTS.md 索引");
      await refresh();
    } catch (error) {
      setStatus(error instanceof Error ? error.message : String(error));
    }
  };

  const remove = async (entry: KnowledgeEntry) => {
    try {
      await bridge.deleteKnowledgeEntry(kind, entry.name);
      setStatus("已删除");
      await refresh();
    } catch (error) {
      setStatus(error instanceof Error ? error.message : String(error));
    }
  };

  const register = async () => {
    try {
      const registration = await bridge.registerKnowledgeWorkspace();
      setStatus(
        `已注册工作区：${registration.workspacePath}（${registration.entryCount} 条）`,
      );
    } catch (error) {
      setStatus(error instanceof Error ? error.message : String(error));
    }
  };

  return (
    <section className="knowledge-panel" data-testid="knowledge-panel" aria-label="知识库">
      <header className="panel-section__header">
        <h2>知识库</h2>
        <button type="button" onClick={() => void register()}>
          注册工作区
        </button>
      </header>
      {root ? <p className="knowledge-panel__root">{root}</p> : null}

      <div className="knowledge-panel__tabs">
        {KINDS.map((item) => (
          <button
            key={item.id}
            type="button"
            className={kind === item.id ? "is-active" : undefined}
            onClick={() => setKind(item.id)}
          >
            {item.label}
          </button>
        ))}
      </div>

      <ul className="knowledge-panel__list">
        {entries.length === 0 ? (
          <li className="panel-section__empty">这个分区还没有内容</li>
        ) : (
          entries.map((entry) => (
            <li key={entry.path}>
              <button type="button" onClick={() => void load(entry)}>
                {entry.name}
              </button>
              <span>{entry.bytes} B</span>
              <button type="button" aria-label="删除" onClick={() => void remove(entry)}>
                ✕
              </button>
            </li>
          ))
        )}
      </ul>

      <div className="knowledge-panel__editor">
        <input
          placeholder="条目名（如：落日配色）"
          value={name}
          onChange={(event) => setName(event.target.value)}
        />
        <textarea
          rows={5}
          placeholder="内容（Markdown / JSON / SVG）"
          value={content}
          onChange={(event) => setContent(event.target.value)}
        />
        <button type="button" onClick={() => void save()} disabled={!name || !content}>
          保存条目
        </button>
      </div>

      {status ? <p className="knowledge-panel__status">{status}</p> : null}
    </section>
  );
}
