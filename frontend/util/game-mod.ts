import { UnifiedMod } from "@hooks/use-unified-mods";

export function getModTitle(mod: UnifiedMod) {
	return mod.merged.title ?? mod.merged.id;
}
