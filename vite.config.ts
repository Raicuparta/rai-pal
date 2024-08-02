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

// https://vitejs.dev/config/
export default defineConfig(async () => ({
	plugins: [react()],
	resolve: {
		alias: getAliasesFromTsconfig(),
	},
	// Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
	//
	// 1. prevent vite from obscuring rust errors
	clearScreen: false,
	// 2. tauri expects a fixed port, fail if that port is not available
	server: {
		port: 1420,
		strictPort: true,
		watch: {
			ignored: ["backend/**"],
		},
	},
	// 3. to make use of `TAURI_DEBUG` and other env variables
	// https://tauri.studio/v1/api/config#buildconfig.beforedevcommand
	envPrefix: [
		"VITE_",
		"TAURI_PLATFORM",
		"TAURI_ARCH",
		"TAURI_FAMILY",
		"TAURI_PLATFORM_VERSION",
		"TAURI_PLATFORM_TYPE",
		"TAURI_DEBUG",
	],
}));
