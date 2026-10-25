import { describe, expect, it } from "vitest";
import { filters as fabricFilters } from "fabric";

import {
  adjustColorMatrix,
  DEFAULT_SPECS,
  describeFilter,
  hasFilters,
  sharpenMatrix,
  toFabricFilters,
} from "./filters";

describe("滤镜映射", () => {
  it("锐化核随强度变化，且中心权重恒为 1+4a", () => {
    const matrix = sharpenMatrix(1);
    expect(matrix).toHaveLength(9);
    expect(matrix[4]).toBeCloseTo(5);
    expect(matrix[1]).toBeCloseTo(-1);
    expect(matrix[0]).toBe(0);
  });

  it("锐化强度被夹在 0..4", () => {
    expect(sharpenMatrix(-3)[1]).toBeCloseTo(0);
    expect(sharpenMatrix(99)[1]).toBeCloseTo(-4);
  });

  it("调色矩阵是 4x5 的数值矩阵", () => {
    const matrix = adjustColorMatrix(0.2, 0.1, -0.3, 90);
    expect(matrix).toHaveLength(20);
    expect(matrix.every((value) => Number.isFinite(value))).toBe(true);
  });

  it("非法数值不会产生 NaN", () => {
    const matrix = adjustColorMatrix(Number.NaN, Number.POSITIVE_INFINITY, 0, 0);
    expect(matrix.every((value) => Number.isFinite(value))).toBe(true);
  });

  it("每种 spec 都能生成对应的 Fabric 滤镜", () => {
    const filters = toFabricFilters([
      DEFAULT_SPECS.blur,
      DEFAULT_SPECS.sharpen,
      DEFAULT_SPECS.adjustColor,
    ]);

    expect(filters).toHaveLength(3);
    expect(filters[0]).toBeInstanceOf(fabricFilters.Blur);
    expect(filters[1]).toBeInstanceOf(fabricFilters.Convolute);
    expect(filters[2]).toBeInstanceOf(fabricFilters.ColorMatrix);
  });

  it("空列表不生成滤镜", () => {
    expect(toFabricFilters([])).toEqual([]);
    expect(hasFilters([])).toBe(false);
    expect(hasFilters([DEFAULT_SPECS.blur])).toBe(true);
  });

  it("describeFilter 给出中文短标签", () => {
    expect(describeFilter(DEFAULT_SPECS.blur)).toContain("模糊");
    expect(describeFilter(DEFAULT_SPECS.sharpen)).toContain("锐化");
    expect(describeFilter(DEFAULT_SPECS.adjustColor)).toContain("调色");
  });
});
