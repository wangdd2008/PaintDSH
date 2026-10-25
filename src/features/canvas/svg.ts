/**
 * SVG 内容操作：图层的矢量内容统一存成一段 `<svg>…</svg>` 字符串。
 *
 * 纯函数、无 Fabric 依赖，方便单测（见 `svg.test.ts`）。
 */

const EMPTY_SVG = '<svg xmlns="http://www.w3.org/2000/svg"></svg>';

export function emptySvg(): string {
  return EMPTY_SVG;
}

/**
 * 把 fragment（如 `<circle .../>`）包装成完整 svg 文档；已经是 `<svg>` 的原样返回。
 */
export function normalizeSvg(content: string): string {
  const trimmed = content.trim();
  if (trimmed.length === 0) {
    return EMPTY_SVG;
  }
  if (/^<svg[\s>]/i.test(trimmed)) {
    return trimmed;
  }
  return `<svg xmlns="http://www.w3.org/2000/svg">${trimmed}</svg>`;
}

/**
 * 往图层 SVG 末尾追加一个元素（画在现有内容之上）。
 * 不是完整 svg 时先包装；找不到 `</svg>` 时退化成整体追加。
 */
export function appendElement(svg: string, element: string): string {
  const document = normalizeSvg(svg);
  const closing = document.lastIndexOf("</svg>");
  if (closing < 0) {
    return normalizeSvg(`${document}${element}`);
  }
  return `${document.slice(0, closing)}${element}${document.slice(closing)}`;
}

/** 取出 `<svg>` 里的子元素数量（用于测试与统计，不做完整解析）。 */
export function countElements(svg: string): number {
  const inner = innerContent(svg);
  if (inner.trim().length === 0) {
    return 0;
  }
  return (inner.match(/<(?!\/)[a-zA-Z]/g) ?? []).length;
}

/** 取 `<svg>` 内部内容；不是完整 svg 时返回原串。 */
export function innerContent(svg: string): string {
  const trimmed = svg.trim();
  const open = trimmed.indexOf(">");
  const close = trimmed.lastIndexOf("</svg>");
  if (!/^<svg[\s>]/i.test(trimmed) || open < 0 || close < 0) {
    return trimmed;
  }
  return trimmed.slice(open + 1, close);
}

/** 内联 SVG → data URL（导出 SVG 时用，避免外部引用）。 */
export function svgToDataUrl(svg: string): string {
  const encoded = encodeURIComponent(normalizeSvg(svg));
  return `data:image/svg+xml;charset=utf-8,${encoded}`;
}

/** 把 #rgb / #rrggbb / 具名颜色统一成 #rrggbb（供形状与文字默认色）。 */
export function normalizeColor(color: string, fallback = "#1c2024"): string {
  const value = color.trim();
  if (/^#[0-9a-f]{6}$/i.test(value)) {
    return value.toLowerCase();
  }
  if (/^#[0-9a-f]{3}$/i.test(value)) {
    const [, r, g, b] = value;
    return `#${r}${r}${g}${g}${b}${b}`.toLowerCase();
  }
  return fallback;
}

/** 生成形状元素（矩形/椭圆），坐标以画布左上角为原点。 */
export function shapeElement(
  shape: "rect" | "ellipse",
  x: number,
  y: number,
  width: number,
  height: number,
  options: { fill?: string; stroke?: string; strokeWidth?: number } = {},
): string {
  const fill = normalizeColor(options.fill ?? "#2f6feb");
  const stroke = options.stroke ? normalizeColor(options.stroke) : "none";
  const strokeWidth = options.strokeWidth ?? 0;
  const left = Math.round(Math.min(x, x + width));
  const top = Math.round(Math.min(y, y + height));
  const w = Math.round(Math.abs(width));
  const h = Math.round(Math.abs(height));

  if (shape === "ellipse") {
    const cx = left + w / 2;
    const cy = top + h / 2;
    return `<ellipse cx="${cx}" cy="${cy}" rx="${w / 2}" ry="${h / 2}" fill="${fill}" stroke="${stroke}" stroke-width="${strokeWidth}"/>`;
  }
  return `<rect x="${left}" y="${top}" width="${w}" height="${h}" fill="${fill}" stroke="${stroke}" stroke-width="${strokeWidth}"/>`;
}

/** 生成文字元素。 */
export function textElement(
  x: number,
  y: number,
  text: string,
  options: { fill?: string; fontSize?: number } = {},
): string {
  const fill = normalizeColor(options.fill ?? "#1c2024");
  const fontSize = options.fontSize ?? 24;
  return `<text x="${Math.round(x)}" y="${Math.round(y)}" fill="${fill}" font-size="${fontSize}" font-family="system-ui, sans-serif">${escapeXml(text)}</text>`;
}

export function escapeXml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&apos;");
}
