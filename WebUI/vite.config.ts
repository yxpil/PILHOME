import { defineConfig } from "vite";

// 纯静态构建:产物输出到 dist,由 Server 端托管。
export default defineConfig({
  base: "./",
  build: {
    outDir: "dist",
    target: "es2022",
  },
  server: {
    proxy: {
      // 开发期代理到本地网关,避免跨域。
      "/api": "http://127.0.0.1:8080",
    },
  },
});