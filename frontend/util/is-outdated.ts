import { GameMod } from "@api/bindings";

export function getIsOutdated(localVersion?: GameMod, remoteVersion?: GameMod) {
	if (!localVersion || !remoteVersion) return false;

	return (
		localVersion.latestVersion.id !== remoteVersion.latestVersion.id ||
		localVersion.hash !== remoteVersion.hash
	);
}
