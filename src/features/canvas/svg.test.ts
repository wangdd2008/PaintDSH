import { describe, expect, it } from "vitest";

import {
  appendElement,
  countElements,
  emptySvg,
  escapeXml,
  innerContent,
  normalizeColor,
  normalizeSvg,
  shapeElement,
  svgToDataUrl,
  textElement,
} from "./svg";

describe("svg helpers", () => {
  it("空白模板是一个可解析的 svg 文档", () => {
    expect(emptySvg().startsWith("<svg")).toBe(true);
    expect(emptySvg().endsWith("</svg>")).toBe(true);
    expect(countElements(emptySvg())).toBe(0);
  });

  it("fragment 会被包装成完整 svg，已经是 svg 的原样返回", () => {
    const wrapped = normalizeSvg('<circle cx="1" cy="2" r="3" />');
    expect(wrapped.startsWith("<svg")).toBe(true);
    expect(wrapped).toContain("<circle");

    const already = '<svg xmlns="http://www.w3.org/2000/svg"><rect /></svg>';
    expect(normalizeSvg(already)).toBe(already);
    expect(normalizeSvg("   ")).toBe(emptySvg());
  });

  it("appendElement 把新元素放在现有内容之后", () => {
    const base = normalizeSvg('<rect x="0" y="0" width="10" height="10" />');
    const next = appendElement(base, '<circle r="5" />');

    expect(countElements(next)).toBe(2);
    const inner = innerContent(next);
    expect(inner.indexOf("<rect")).toBeLessThan(inner.indexOf("<circle"));
    expect(next.endsWith("</svg>")).toBe(true);
  });

  it("appendElement 对空白模板也能工作", () => {
    const next = appendElement(emptySvg(), '<ellipse rx="4" ry="2" />');
    expect(countElements(next)).toBe(1);
  });

  it("circle 与矩形共享同一坐标原点（画布左上角）", () => {
    const rect = shapeElement("rect", 120, 80, 40, 30);
    expect(rect).toContain('x="120"');
    expect(rect).toContain('y="80"');
    expect(rect).toContain('width="40"');
    expect(rect).toContain('height="30"');
  });

  it("反向拖动矩形也能得到正的宽高", () => {
    const rect = shapeElement("rect", 100, 100, -40, -20);
    expect(rect).toContain('x="60"');
    expect(rect).toContain('y="80"');
    expect(rect).toContain('width="40"');
    expect(rect).toContain('height="20"');
  });

  it("椭圆根据包围盒换算中心与半径", () => {
    const ellipse = shapeElement("ellipse", 0, 0, 100, 50);
    expect(ellipse).toContain('cx="50"');
    expect(ellipse).toContain('cy="25"');
    expect(ellipse).toContain('rx="50"');
    expect(ellipse).toContain('ry="25"');
  });

  it("颜色统一成六位小写十六进制，非法值回落", () => {
    expect(normalizeColor("#FFF")).toBe("#ffffff");
    expect(normalizeColor("#E5484D")).toBe("#e5484d");
    expect(normalizeColor("red")).toBe("#1c2024");
    expect(normalizeColor("red", "#000000")).toBe("#000000");
  });

  it("文字元素转义 XML 特殊字符", () => {
    const element = textElement(10, 20, 'a<b&c"d');
    expect(element).toContain("a&lt;b&amp;c&quot;d");
    expect(escapeXml("'")).toBe("&apos;");
  });

  it("svgToDataUrl 生成可用的 svg data URL", () => {
    const url = svgToDataUrl('<circle r="1" />');
    expect(url.startsWith("data:image/svg+xml;charset=utf-8,")).toBe(true);
    expect(decodeURIComponent(url.split(",")[1])).toContain("<circle");
  });
});
