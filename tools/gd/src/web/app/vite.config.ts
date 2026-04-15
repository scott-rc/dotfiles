import { defineConfig } from "vite";
import preact from "@preact/preset-vite";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [preact(), tailwindcss()],
  resolve: {
    alias: {
      react: "preact/compat",
      "react-dom": "preact/compat",
      "react-dom/test-utils": "preact/test-utils",
      "react/jsx-runtime": "preact/jsx-runtime",
    },
  },
  server: {
    proxy: {
      "/ws": {
        target: "ws://localhost:3845",
        ws: true,
      },
    },
  },
  build: {
    outDir: "dist",
  },
});
