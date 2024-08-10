import { Store } from "@tauri-apps/plugin-store";

export function getDataCache() {
	return new Store(".data-cache.dat");
}

export function clearDataCache() {
	const cache = getDataCache();
	cache.reset();
	cache.save();
}
