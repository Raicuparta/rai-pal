import { Store } from "@tauri-apps/plugin-store";

export function getDataCache() {
	return new Store(".data-cache.dat");
}

export async function clearDataCache() {
	const cache = getDataCache();
	await cache.reset();
	await cache.save();
}
