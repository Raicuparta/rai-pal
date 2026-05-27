import { UnifiedModVersion } from "@hooks/use-unified-mods";

export function getIsOutdated(
	local?: UnifiedModVersion,
	remote?: UnifiedModVersion,
) {
	if (!local || !remote) return false;

	return (
		local.latestVersion?.id !== remote.latestVersion?.id ||
		local.hash !== remote.hash
	);
}
