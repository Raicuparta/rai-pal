import { UnifiedMod } from "@hooks/use-unified-mods";

export function getModTitle(mod: UnifiedMod) {
	return mod.remote?.title ?? mod.local?.manifest?.title ?? mod.common.id;
}
