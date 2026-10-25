/**
 * Tauri IPC 的唯一封装层。
 *
 * 组件里禁止直接 import `@tauri-apps/api`——所有命令与事件都从这里进出，
 * 这样测试可以整体 `vi.mock('./lib/bridge')`（engineering-contract 第 1、8 节）。
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type { PingResponse } from "./types";

export type CommandArgs = Record<string, unknown>;

/** 通用 invoke 包装：返回类型由调用方声明，与 Rust 命令签名对齐。 */
export async function invokeCommand<T>(command: string, args?: CommandArgs): Promise<T> {
  return invoke<T>(command, args);
}

/** 探活：后端在线时返回 `{ pong: true, version }`。 */
export async function ping(): Promise<PingResponse> {
  return invokeCommand<PingResponse>("ping");
}

/** 订阅后端事件，返回取消订阅函数。 */
export async function onEvent<T>(
  event: string,
  handler: (payload: T) => void,
): Promise<UnlistenFn> {
  return listen<T>(event, (event) => {
    handler(event.payload);
  });
}
