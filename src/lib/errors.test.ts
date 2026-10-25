import { describe, expect, it } from "vitest";

import { describeAppError, isAppErrorPayload, parseAppError } from "./errors";

describe("errors", () => {
  it("识别后端返回的 {kind, message} 结构", () => {
    const payload = { kind: "notFound", message: "画布不存在" };

    expect(isAppErrorPayload(payload)).toBe(true);
    expect(parseAppError(payload)).toEqual(payload);
  });

  it("拒绝缺少字段或 kind 非法的对象", () => {
    expect(isAppErrorPayload({ kind: "notFound" })).toBe(false);
    expect(isAppErrorPayload({ kind: "boom", message: "x" })).toBe(false);
    expect(isAppErrorPayload(null)).toBe(false);
  });

  it("字符串错误退化为 unknown，并保留原始消息", () => {
    expect(parseAppError("窗口未就绪")).toEqual({ kind: "unknown", message: "窗口未就绪" });
  });

  it("Error 实例取其 message", () => {
    expect(parseAppError(new Error("网络断开"))).toEqual({
      kind: "unknown",
      message: "网络断开",
    });
  });

  it("已知错误类型给出中文描述", () => {
    expect(describeAppError({ kind: "revisionConflict", message: "revision 5 != 6" })).toBe(
      "版本冲突：revision 5 != 6",
    );
    expect(describeAppError("窗口未就绪")).toBe("未知错误：窗口未就绪");
  });
});
