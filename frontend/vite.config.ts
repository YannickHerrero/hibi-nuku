import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import { VitePWA } from "vite-plugin-pwa";
import path from "node:path";

const BACKEND = process.env.NUKU_BACKEND_URL ?? "http://localhost:8787";

export default defineConfig({
  plugins: [
    tanstackRouter({ target: "react", autoCodeSplitting: true }),
    react(),
    tailwindcss(),
    VitePWA({
      registerType: "autoUpdate",
      manifest: {
        name: "Hibi Nuku",
        short_name: "Nuku",
        description: "日々抜く — Japanese sentence mining",
        theme_color: "#b5593a",
        background_color: "#f4ebd9",
        display: "standalone",
        start_url: "/",
        icons: [],
      },
      workbox: {
        navigateFallback: "/index.html",
        // Don't precache the giant dict bundles; we cache them via a
        // runtime route below.
        globPatterns: ["**/*.{js,css,html,ico,svg,woff,woff2}"],
        runtimeCaching: [
          {
            urlPattern: /\/api\/dict\//,
            handler: "CacheFirst",
            options: {
              cacheName: "nuku-dict",
              expiration: { maxEntries: 6 },
            },
          },
          {
            urlPattern: /\/api\//,
            handler: "NetworkFirst",
            options: {
              cacheName: "nuku-api",
              networkTimeoutSeconds: 5,
            },
          },
        ],
      },
    }),
  ],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  server: {
    host: true,            // bind to 0.0.0.0 so the tailnet can reach it
    port: 5173,
    strictPort: true,
    // Tailnet hostnames; extend if you connect from a new device.
    allowedHosts: ["ilios", "100.89.150.38", ".ts.net"],
    proxy: {
      "/api": {
        target: BACKEND,
        changeOrigin: true,
      },
    },
  },
});
