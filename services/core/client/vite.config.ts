import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";
import tsconfigPaths from "vite-tsconfig-paths";
import { VitePWA } from "vite-plugin-pwa";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

export default defineConfig({
    define: {
        "import.meta.env.GIT_COMMIT": JSON.stringify(process.env.GIT_COMMIT ?? ""),
        "import.meta.env.LAST_UPDATED": JSON.stringify(process.env.LAST_UPDATED ?? ""),
    },
    resolve: {
        alias: {
            "@": path.resolve(__dirname, "./src"),
        },
    },
    plugins: [
        tsconfigPaths(),
        react(),
        tailwindcss(),
        VitePWA({
            registerType: "autoUpdate",
            devOptions: {
                enabled: false,
            },
            includeAssets: [
                "favicon.ico",
                "pwa-64x64.png",
                "pwa-192x192.png",
                "pwa-512x512.png",
                "apple-touch-icon-180x180.png",
                "maskable-icon-512x512.png",
            ],
            manifest: {
                name: "Webapp Template",
                short_name: "Webapp Template",
                description: "Webapp Template",
                theme_color: "#ffffff",
                icons: [
                    {
                        src: "pwa-64x64.png",
                        sizes: "64x64",
                        type: "image/png",
                    },
                    {
                        src: "pwa-192x192.png",
                        sizes: "192x192",
                        type: "image/png",
                    },
                    {
                        src: "pwa-512x512.png",
                        sizes: "512x512",
                        type: "image/png",
                    },
                    {
                        src: "maskable-icon-512x512.png",
                        sizes: "512x512",
                        type: "image/png",
                        purpose: "maskable",
                    },
                ],
                screenshots: [
                    {
                        src: "pwa-512x512.png",
                        sizes: "512x512",
                        type: "image/png",
                        form_factor: "wide",
                        label: "SC1",
                    },
                    {
                        src: "pwa-512x512.png",
                        sizes: "512x512",
                        type: "image/png",
                        form_factor: "narrow",
                        label: "SC2",
                    },
                ],
            },
        }),
    ],
});
