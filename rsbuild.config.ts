import { defineConfig } from "@rsbuild/core";
import { pluginReact } from "@rsbuild/plugin-react";
import { pluginSvgr } from "@rsbuild/plugin-svgr";

export default defineConfig({
  output: {
    distPath: {
      root: "./dist/web",
    },
    // Ensure relative paths for Tauri to load assets correctly via custom protocol
    assetPrefix: "./",
    cleanDistPath: true,
  },
  performance: {
    removeMomentLocale: true,
  },
  html: {
    template: "./src/index.html",
  },
  plugins: [
    pluginReact(),
    pluginSvgr({
      svgrOptions: {
        exportType: "named",
        svgo: true,
        svgoConfig: {
          plugins: [
            {
              name: "preset-default",
              params: { overrides: { removeViewBox: false } },
            },
          ],
        },
      },
    }),
  ],
  dev: {
    writeToDisk: true,
    lazyCompilation: false,
    cliShortcuts: false,
    assetPrefix: "http://localhost:5173/",
  },
  server: {
    port: 5678,
    strictPort: false,
    printUrls: false,
    open: false,
    compress: false,
  },
});
