import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Konfiguracja zgodna z oczekiwaniami Tauri (stały port dev, brak czyszczenia ekranu).
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  envPrefix: ["VITE_", "TAURI_"],
});
