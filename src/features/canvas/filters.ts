/**
 * 滤镜声明 → Fabric.js filter 实例的映射（纯函数，便于单测）。
 *
 * 滤镜是**声明式元数据**：Rust 只存 `FilterSpec`，实际像素效果由前端按 spec 重绘
 * （engineering-contract 第 3 节）。
 */

import { filters } from "fabric";

import type { FilterSpec } from "../../lib/types";

export const DEFAULT_SPECS: Record<FilterSpec["kind"], FilterSpec> = {
  blur: { kind: "blur", radius: 6 },
  sharpen: { kind: "sharpen", amount: 1.2 },
  adjustColor: {
    kind: "adjustColor",
    brightness: 0,
    contrast: 0.1,
    saturation: 0.1,
    hue: 0,
  },
};

/** 锐化卷积核（中心 1 + 4·amount，四邻域 -amount）。 */
export function sharpenMatrix(amount: number): number[] {
  const a = Math.min(Math.max(amount, 0), 4);
  return [0, -a, 0, -a, 1 + 4 * a, -a, 0, -a, 0];
}

/** 明度/对比度/饱和度/色相的近似 ColorMatrix（Fabric 无内置 hue 旋转）。 */
export function adjustColorMatrix(
  brightness: number,
  contrast: number,
  saturation: number,
  hue: number,
): number[] {
  const b = 1 + clamp(brightness, -1, 1);
  const c = 1 + clamp(contrast, -1, 1);
  const s = 1 + clamp(saturation, -1, 1);
  // 色相用 RGB→YIQ 的近似旋转（每 120° 一个通道轮换，缩放 hue 到 0..1）。
  const h = ((hue % 360) + 360) % 360;
  const rad = (h * Math.PI) / 180;
  const cos = Math.cos(rad);
  const sin = Math.sin(rad);

  const saturationMatrix = [
    0.213 + 0.787 * s - 0.213 * s,
    0.715 - 0.715 * s,
    0.072 - 0.072 * s,
    0.213 - 0.213 * s,
    0.715 + 0.285 * s - 0.715 * s,
    0.072 - 0.072 * s,
    0.213 - 0.213 * s,
    0.715 - 0.715 * s,
    0.072 + 0.928 * s,
  ];

  // 简化色相旋转：混合原图与 cos/sin 加权，够用即可（不做完整色彩空间变换）。
  const hueMatrix = [
    0.213 + cos * 0.787 - sin * 0.213,
    0.715 - cos * 0.715 - sin * 0.715,
    0.072 - cos * 0.072 + sin * 0.928,
    0.213 - cos * 0.213 + sin * 0.143,
    0.715 + cos * 0.285 + sin * 0.14,
    0.072 - cos * 0.072 - sin * 0.283,
    0.213 - cos * 0.213 - sin * 0.787,
    0.715 - cos * 0.715 + sin * 0.715,
    0.072 + cos * 0.928 + sin * 0.072,
  ];

  const mixed = saturationMatrix.map(
    (value, index) => value * 0.5 + hueMatrix[index] * 0.5,
  );

  return [
    mixed[0] * c * b,
    mixed[1] * c * b,
    mixed[2] * c * b,
    0,
    0,
    mixed[3] * c * b,
    mixed[4] * c * b,
    mixed[5] * c * b,
    0,
    0,
    mixed[6] * c * b,
    mixed[7] * c * b,
    mixed[8] * c * b,
    0,
    0,
    0,
    0,
    0,
    1,
    0,
  ];
}

function clamp(value: number, low: number, high: number): number {
  if (!Number.isFinite(value)) {
    return 0;
  }
  return Math.min(Math.max(value, low), high);
}

/** Fabric 滤镜实例（只用到这三种）。 */
export type FabricFilter =
  | InstanceType<typeof filters.Blur>
  | InstanceType<typeof filters.Convolute>
  | InstanceType<typeof filters.ColorMatrix>;

/** 把声明式 spec 转成 Fabric filter 列表（顺序即应用顺序）。 */
export function toFabricFilters(specs: FilterSpec[]): FabricFilter[] {
  return specs.map((spec) => {
    switch (spec.kind) {
      case "blur":
        return new filters.Blur({ blur: clamp(spec.radius, 0, 100) / 100 });
      case "sharpen":
        return new filters.Convolute({
          matrix: sharpenMatrix(spec.amount),
          opaque: true,
        });
      case "adjustColor":
        return new filters.ColorMatrix({
          matrix: adjustColorMatrix(
            spec.brightness,
            spec.contrast,
            spec.saturation,
            spec.hue,
          ),
        });
    }
  });
}

/** 是否存在会改变像素的滤镜。 */
export function hasFilters(specs: FilterSpec[]): boolean {
  return specs.length > 0;
}

/** 滤镜的中文短标签（面板里显示）。 */
export function describeFilter(spec: FilterSpec): string {
  switch (spec.kind) {
    case "blur":
      return `模糊 ${spec.radius}`;
    case "sharpen":
      return `锐化 ${spec.amount}`;
    case "adjustColor":
      return `调色 亮度${spec.brightness} 对比${spec.contrast} 饱和${spec.saturation} 色相${spec.hue}`;
  }
}
