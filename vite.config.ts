import { defineConfig } from "vite";

import { fileURLToPath, URL } from "node:url";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,

  // Segunda página independiente: la ventana overlay de captura de
  // coordenada (WebviewUrl::App("captura.html")) — sin esto Vite
  // solo compila index.html y esa ventana carga en blanco (404).
  // Tercera página independiente: la ventana overlay de MenuExpress
  // (WebviewUrl::App("menu_express.html?id=...") — ver
  // back_menu_express.rs). Mismo motivo que captura: sin esto Vite
  // no la compila y la ventana carga en blanco (404).
  build: {
    rollupOptions: {
      input: {
        main: fileURLToPath(new URL("./index.html", import.meta.url)),
        captura: fileURLToPath(new URL("./captura.html", import.meta.url)),
        menuExpress: fileURLToPath(
          new URL("./menu_express.html", import.meta.url),
        ),
      },
    },
  },

  // 2. tauri expects a fixed port, fail if that port is not available
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
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
