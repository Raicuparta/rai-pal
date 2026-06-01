import { GameMod } from "@api/bindings";

export function getModTitle(mod: GameMod) {
	return mod.title ?? mod.id;
}
