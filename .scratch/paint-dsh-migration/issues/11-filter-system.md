# 11: 滤镜系统

**What to build:**

用户可对图层应用模糊、锐化、调色等滤镜效果。

**Blocked by:** 03

**Status:** resolved

- [x] 滤镜算法正确（模糊、锐化、调色）
- [x] 前端滤镜面板可用
- [x] 滤镜效果正确应用到图层

## Comments

- 提交：`cc78fdf`（`FilterSpec` + `apply_filter` / `clear_filters`）、`4479512`（面板与渲染）。
- 架构取舍：滤镜存成**声明式元数据**（`layer.metadata.filters`），Rust 只校验范围并落快照，像素效果由前端按 spec 重绘（`fabric.filters.Blur` / `Convolute` / `ColorMatrix`）。好处：零像素处理依赖、滤镜可 undo/redo、导出自动带上效果。
- 算法：亮度/对比度/饱和度用 3×3 混合矩阵 + 缩放，色相用 YIQ 近似旋转矩阵；锐化用 5 点卷积核 `[0,-a,0,-a,1+4a,-a,0,-a,0]`，`a` 夹在 0..4；模糊把像素半径换算成 Fabric 的 0..1 比例。
- 限制：只允许 raster 图层直接加滤镜（矢量/aiCode 会被拒绝并提示先栅格化），因为 Fabric 的滤镜只能作用在图像对象上。
- 验证：`filters.test.ts` 7 个用例（锐化核数值与夹取、调色矩阵 4×5 且无 NaN、三种 spec 生成对应 Fabric 类、空列表、中文标签）；Rust 侧 `apply_filter` 的非法值/图层类型/清空共 3 个用例。
