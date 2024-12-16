import { GameTag, OwnedGame } from "@api/bindings";

export function filterGameTags(
	ownedGame: OwnedGame | undefined,
	hiddenValues: (string | null)[],
) {
	return (
		hiddenValues.findIndex(
			(hiddenValue) => ownedGame?.tags.indexOf(hiddenValue as GameTag) !== -1,
		) !== -1
	);
}

export function getGameTagsSortValue(ownedGame: OwnedGame | null) {
	return ownedGame
		? `${ownedGame.tags.length}${ownedGame.tags.sort().join(",")}`
		: "";
}
