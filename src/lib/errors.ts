/**
 * 与 Rust `AppError` / `ErrorKind` 一一对应的前端错误类型。
 * 后端序列化为 `{ kind, message }`（`kind` 为 camelCase），见 engineering-contract 第 1、6 节。
 */

export const KNOWN_ERROR_KINDS = [
  "revisionConflict",
  "notFound",
  "invalidAction",
  "io",
  "db",
  "ai",
  "harness",
  "timeout",
  "busy",
] as const;

export type KnownErrorKind = (typeof KNOWN_ERROR_KINDS)[number];

/** 后端未按约定返回时统一归为 `unknown`，保证 UI 永远能显示一句话。 */
export type AppErrorKind = KnownErrorKind | "unknown";

export interface AppErrorPayload {
  kind: AppErrorKind;
  message: string;
}

const ERROR_KIND_LABELS: Record<AppErrorKind, string> = {
  revisionConflict: "版本冲突",
  notFound: "资源不存在",
  invalidAction: "操作不合法",
  io: "文件读写失败",
  db: "数据库错误",
  ai: "AI 服务错误",
  harness: "DSH 执行失败",
  timeout: "操作超时",
  busy: "系统忙",
  unknown: "未知错误",
};

function isKnownKind(value: unknown): value is KnownErrorKind {
  return typeof value === "string" && (KNOWN_ERROR_KINDS as readonly string[]).includes(value);
}

/** 判断任意值是否形如后端错误载荷。 */
export function isAppErrorPayload(value: unknown): value is AppErrorPayload {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const candidate = value as { kind?: unknown; message?: unknown };
  return isKnownKind(candidate.kind) && typeof candidate.message === "string";
}

/** 把 invoke 抛出的任意错误规整成 `{ kind, message }`。 */
export function parseAppError(error: unknown): AppErrorPayload {
  if (isAppErrorPayload(error)) {
    return { kind: error.kind, message: error.message };
  }
  if (typeof error === "string") {
    return { kind: "unknown", message: error };
  }
  if (error instanceof Error) {
    return { kind: "unknown", message: error.message };
  }
  if (typeof error === "object" && error !== null) {
    const candidate = error as { kind?: unknown; message?: unknown };
    if (typeof candidate.message === "string") {
      return {
        kind: isKnownKind(candidate.kind) ? candidate.kind : "unknown",
        message: candidate.message,
      };
    }
  }
  return { kind: "unknown", message: String(error) };
}

/** 面向用户的中文错误描述，例如「版本冲突：revision 5 != 6」。 */
export function describeAppError(error: unknown): string {
  const { kind, message } = parseAppError(error);
  return `${ERROR_KIND_LABELS[kind]}：${message}`;
}
