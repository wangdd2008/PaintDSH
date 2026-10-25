import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri 期望固定端口；dev host 由 TAURI_DEV_HOST 注入（移动端调试用）。
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react()],
  // 只有 VITE_ 前缀会暴露给前端；TAURI_ 前缀用于 Tauri 自身注入的变量。
  envPrefix: ["VITE_", "TAURI_"],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    // Windows 上 WebView2 / macOS WKWebView 都能吃 ES2021，避免过旧的降级。
    target: process.env.TAURI_ENV_PLATFORM === "windows" ? "chrome105" : "safari15",
    minify: process.env.TAURI_ENV_DEBUG ? false : "esbuild",
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
    // Fabric.js + React 打成一个 chunk 会超过默认 500kB 阈值，桌面端本地加载无需拆包。
    chunkSizeWarningLimit: 900,
  },
});
