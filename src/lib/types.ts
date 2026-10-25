/**
 * 与 Rust DTO 一一对应的手写 TS 类型（无生成器）。
 * 本工单只落地 IPC 骨架所需的类型；`CanvasDocument` / `Layer` 等随 ticket 02 补齐。
 */

export interface PingResponse {
  pong: boolean;
  version: string;
}

export interface Size {
  width: number;
  height: number;
}

export interface Region extends Size {
  x: number;
  y: number;
}

export type Actor = "user" | "agent";
