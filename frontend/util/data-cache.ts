import { Store } from "@tauri-apps/plugin-store";

export function getDataCache() {
	return new Store(".data-cache.dat");
}
