import { beforeEach, describe, expect, it, vi } from "vitest";

const tauri = vi.hoisted(() => ({
  invoke: vi.fn(),
  listen: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: tauri.invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen: tauri.listen }));

import { invokeCommand, onEvent, ping } from "./bridge";

beforeEach(() => {
  tauri.invoke.mockReset();
  tauri.listen.mockReset();
});

describe("bridge", () => {
  it("ping 调用后端的 ping 命令并返回版本信息", async () => {
    tauri.invoke.mockResolvedValue({ pong: true, version: "0.1.0" });

    await expect(ping()).resolves.toEqual({ pong: true, version: "0.1.0" });
    expect(tauri.invoke).toHaveBeenCalledWith("ping", undefined);
  });

  it("invokeCommand 透传命令名与 camelCase 参数", async () => {
    tauri.invoke.mockResolvedValue({ ok: true });

    await invokeCommand("apply_action", { canvasId: "c-1", action: "undo" });

    expect(tauri.invoke).toHaveBeenCalledWith("apply_action", {
      canvasId: "c-1",
      action: "undo",
    });
  });

  it("onEvent 把 event.payload 交给回调，并返回取消订阅函数", async () => {
    const unlisten = vi.fn();
    tauri.listen.mockResolvedValue(unlisten);
    const received: unknown[] = [];

    const stop = await onEvent<{ revision: number }>("canvas://changed", (payload) => {
      received.push(payload);
    });

    const handler = tauri.listen.mock.calls[0]![1] as (e: { payload: unknown }) => void;
    handler({ payload: { revision: 7 } });

    expect(tauri.listen).toHaveBeenCalledWith("canvas://changed", expect.any(Function));
    expect(received).toEqual([{ revision: 7 }]);
    expect(stop).toBe(unlisten);
  });
});
