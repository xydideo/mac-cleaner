import { defineConfig } from "vite"
import vue from "@vitejs/plugin-vue"
import path from "path"
import ElementPlus from "unplugin-element-plus/vite"

export default defineConfig({
  plugins: [
    vue(),
    ElementPlus({
      useSource: true,
    }),
  ],
  base: "./",
  build: {
    outDir: "dist",
  },
  clearScreen: false,
  server: {
    port: 1421,
    strictPort: true,
    host: false,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  envPrefix: ["VITE_", "TAURI_"],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  css: {
    preprocessorOptions: {
      scss: {
        additionalData: `@use "${path.resolve(__dirname, "src/assets/styles/variables.scss")}" as *;`,
      },
    },
  },
})
