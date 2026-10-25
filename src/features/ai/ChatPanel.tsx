import { useCallback, useEffect, useRef, useState } from "react";

import * as bridge from "../../lib/bridge";
import type { AiDelta, AiDone, AiError, AiPendingOp, Employee } from "../../lib/types";
import { usePaintStore } from "../../stores/paint";

interface ChatMessage {
  id: string;
  role: "user" | "assistant" | "system";
  content: string;
  reasoning?: string;
  pending?: boolean;
  meta?: string;
}

/**
 * AI 对话面板：Direct 引擎的流式对话 + Harness 任务入口。
 *
 * 事件（ai://delta / done / error / pending-op）由 Rust 推送，这里只负责累积与展示；
 * 画布改动由后端 paint-ops 直接走 dispatcher，面板不重复实现。
 */
export function ChatPanel() {
  const document = usePaintStore((state) => state.document);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState("");
  const [engine, setEngine] = useState<"auto" | "harness" | "direct">("auto");
  const [employeeId, setEmployeeId] = useState<string>("");
  const [employees, setEmployees] = useState<Employee[]>([]);
  const [running, setRunning] = useState(false);
  const [tokens, setTokens] = useState({ input: 0, output: 0 });
  const [error, setError] = useState<string | null>(null);
  const sessionRef = useRef<string | null>(null);

  useEffect(() => {
    void bridge
      .listEmployees()
      .then(setEmployees)
      .catch(() => setEmployees([]));
  }, []);

  const appendDelta = useCallback((payload: AiDelta) => {
    if (sessionRef.current && payload.sessionId !== sessionRef.current) {
      return;
    }
    setMessages((current) => {
      const next = [...current];
      const last = next[next.length - 1];
      if (!last || last.role !== "assistant") {
        return [
          ...next,
          {
            id: payload.sessionId,
            role: "assistant",
            content: payload.kind === "content" ? payload.content : "",
            reasoning: payload.kind === "reasoning" ? payload.content : undefined,
            pending: true,
          },
        ];
      }
      if (payload.kind === "content") {
        last.content += payload.content;
      } else {
        last.reasoning = `${last.reasoning ?? ""}${payload.content}`;
      }
      return next;
    });
  }, []);

  useEffect(() => {
    const unlisteners: Array<() => void> = [];

    void bridge
      .onEvent<AiDelta>("ai://delta", appendDelta)
      .then((unlisten) => unlisteners.push(unlisten));
    void bridge
      .onEvent<AiDone>("ai://done", (payload) => {
        setRunning(false);
        setTokens({ input: payload.tokensInput, output: payload.tokensOutput });
        setMessages((current) =>
          current.map((message) =>
            message.pending
              ? {
                  ...message,
                  pending: false,
                  meta: `${payload.engine} · ${payload.model} · ${payload.tokensInput}/${payload.tokensOutput} tokens · ${payload.latencyMs}ms`,
                }
              : message,
          ),
        );
      })
      .then((unlisten) => unlisteners.push(unlisten));
    void bridge
      .onEvent<AiError>("ai://error", (payload) => {
        setRunning(false);
        setError(`${payload.kind}: ${payload.message}`);
        setMessages((current) =>
          current.map((message) =>
            message.pending ? { ...message, pending: false } : message,
          ),
        );
      })
      .then((unlisten) => unlisteners.push(unlisten));
    void bridge
      .onEvent<AiPendingOp>("ai://pending-op", (payload) => {
        setMessages((current) => [
          ...current,
          {
            id: `pending-${Date.now()}`,
            role: "system",
            content: `AI 想执行「${payload.action}」，需要你确认：${payload.reason}`,
          },
        ]);
      })
      .then((unlisten) => unlisteners.push(unlisten));

    return () => {
      for (const unlisten of unlisteners) {
        unlisten();
      }
    };
  }, [appendDelta]);

  const send = async () => {
    const prompt = input.trim();
    if (!prompt || running) {
      return;
    }
    setError(null);
    setInput("");
    setMessages((current) => [
      ...current,
      { id: `user-${Date.now()}`, role: "user", content: prompt },
    ]);

    try {
      const start = await bridge.aiChatStart({
        canvasId: document?.id,
        employeeId: employeeId || undefined,
        prompt,
        engine: engine === "auto" ? undefined : engine,
        module: employeeId ? "employee" : undefined,
      });
      sessionRef.current = start.sessionId;
      setRunning(true);
      setMessages((current) => [
        ...current,
        {
          id: start.sessionId,
          role: "system",
          content: `已交给 ${start.engine} 引擎（${start.model}）处理…`,
        },
      ]);
    } catch (cause) {
      setRunning(false);
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  };

  return (
    <section className="chat-panel" data-testid="chat-panel" aria-label="AI 对话">
      <header className="panel-section__header">
        <h2>AI 助手</h2>
        <select
          aria-label="引擎"
          value={engine}
          onChange={(event) => setEngine(event.target.value as typeof engine)}
        >
          <option value="auto">自动选择</option>
          <option value="harness">Harness</option>
          <option value="direct">Direct</option>
        </select>
      </header>

      <select
        aria-label="数字员工"
        className="chat-panel__employee"
        value={employeeId}
        onChange={(event) => setEmployeeId(event.target.value)}
      >
        <option value="">不使用数字员工</option>
        {employees.map((employee) => (
          <option key={employee.id} value={employee.id}>
            {employee.name}
          </option>
        ))}
      </select>

      <ol className="chat-panel__messages" data-testid="chat-messages">
        {messages.length === 0 ? (
          <li className="chat-panel__empty">
            试试：「帮我画一个红色的圆」或「给当前图层加一点模糊」
          </li>
        ) : null}
        {messages.map((message) => (
          <li key={message.id} className={`chat-message chat-message--${message.role}`}>
            {message.reasoning ? (
              <details>
                <summary>思考过程</summary>
                <pre>{message.reasoning}</pre>
              </details>
            ) : null}
            <p>{message.content || (message.pending ? "…" : "")}</p>
            {message.meta ? <small>{message.meta}</small> : null}
          </li>
        ))}
      </ol>

      {error ? (
        <p className="chat-panel__error" role="alert" data-testid="chat-error">
          {error}
        </p>
      ) : null}

      <div className="chat-panel__composer">
        <textarea
          aria-label="输入指令"
          value={input}
          rows={3}
          placeholder="描述你想画什么，AI 会生成 paint-ops 操作"
          onChange={(event) => setInput(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter" && !event.shiftKey) {
              event.preventDefault();
              void send();
            }
          }}
        />
        <div className="chat-panel__actions">
          <span className="chat-panel__tokens">
            {tokens.input}/{tokens.output} tokens
          </span>
          <button type="button" onClick={() => void send()} disabled={running}>
            {running ? "处理中…" : "发送"}
          </button>
          <button type="button" onClick={() => void bridge.aiCancel()} disabled={!running}>
            取消
          </button>
        </div>
      </div>
    </section>
  );
}
