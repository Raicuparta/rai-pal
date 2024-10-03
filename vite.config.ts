import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

import tsconfig from "./tsconfig.json";

function removeAsterisk(pathWithAsterisk: string) {
	return pathWithAsterisk.replace(/\/\*$/, "");
}

function getAliasesFromTsconfig() {
	const resolveAlias = {};
	for (const [key, [resolvePath]] of Object.entries(
		tsconfig.compilerOptions.paths,
	)) {
		resolveAlias[removeAsterisk(key)] = `/${removeAsterisk(resolvePath)}`;
	}

	return resolveAlias;
}

export default defineConfig(async () => ({
	plugins: [react()],
	resolve: {
		alias: getAliasesFromTsconfig(),
	},
	clearScreen: false,
	server: {
		// Tauri expects a fixed port, fail if that port is not available
		strictPort: true,
		// tauri.conf.json needs to specify the same port, under build.devUrl.
		port: 1420,
		watch: {
			// The target folder has all the backend builds, and gets huge, making the watcher super slow.
			ignored: ["backend/**"],
		},
	},
	build: {
		// tauri.config.json needs to point to the same path, under build.frontendDist.
		outDir: "dist",
		rollupOptions: {
			output: {
				manualChunks: undefined,
			},
		},
		chunkSizeWarningLimit: 2000,
	},
	envPrefix: ["VITE_", "TAURI_ENV_"],
}));
