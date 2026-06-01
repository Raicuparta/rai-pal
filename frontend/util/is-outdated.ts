import { GameMod } from "@api/bindings";

export function getIsOutdated(local?: GameMod, remote?: GameMod) {
	if (!local || !remote) return false;

	return (
		local.download?.id !== remote.download?.id || local.hash !== remote.hash
	);
}
