import { useState } from "react";

import * as bridge from "../../lib/bridge";
import type { DshWebInfo } from "../../lib/types";

/**
 * DSH Web UI：启动本地 web profile，再用独立 WebviewWindow 打开。
 *
 * 不用 iframe —— DSH 的启动 token 换取的是 `SameSite=Strict` cookie，
 * iframe 属第三方上下文，`/api` 与 WebSocket 会全部鉴权失败。
 */
export function HarnessView() {
  const [info, setInfo] = useState<DshWebInfo | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const start = async () => {
    setBusy(true);
    setStatus("正在启动 DSH Web…");
    try {
      const started = await bridge.startDshWeb();
      setInfo(started);
      setStatus(`已就绪：端口 ${started.port}`);
    } catch (error) {
      setStatus(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  };

  const open = async () => {
    try {
      await bridge.openDshWebWindow();
      setStatus("已在新窗口打开（带 PAINT 品牌注入）");
    } catch (error) {
      setStatus(error instanceof Error ? error.message : String(error));
    }
  };

  const stop = async () => {
    try {
      await bridge.stopDshWeb();
      setInfo(null);
      setStatus("已停止");
    } catch (error) {
      setStatus(error instanceof Error ? error.message : String(error));
    }
  };

  return (
    <section className="harness-panel" data-testid="harness-panel" aria-label="DSH Web UI">
      <header className="panel-section__header">
        <h2>DSH 工作台</h2>
      </header>
      <p className="harness-panel__hint">
        headless 负责无人值守任务；完整对话界面在独立窗口里跑（共享 DSH_HOME）。
      </p>
      <div className="harness-panel__actions">
        <button type="button" onClick={() => void start()} disabled={busy || Boolean(info)}>
          启动
        </button>
        <button type="button" onClick={() => void open()} disabled={!info}>
          打开窗口
        </button>
        <button type="button" onClick={() => void stop()} disabled={!info}>
          停止
        </button>
      </div>
      {info ? (
        <p className="harness-panel__info">
          端口 {info.port}，PID {info.pid}（启动 URL 含 token，不在此显示）
        </p>
      ) : null}
      {status ? <p className="harness-panel__status">{status}</p> : null}
    </section>
  );
}
