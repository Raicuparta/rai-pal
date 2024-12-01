import { GameTag, OwnedGame } from "@api/bindings";
import { Table } from "@mantine/core";
import { FilterOption } from "../table/table-head";
import styles from "./game-tags.module.css";

function getTagDisplayName(tag: GameTag) {
	return (
		gameTagFilterOptions.find((option) => option.value === tag)?.label || tag
	);
}

export function renderGameTagsCell(ownedGame: OwnedGame | null) {
	return (
		<Table.Td p={0}>
			<div className={styles.wrapper}>
				{ownedGame?.tags.sort().map((tag) => (
					<span
						className={styles.tag}
						key={tag}
					>
						{getTagDisplayName(tag)}
					</span>
				))}
			</div>
		</Table.Td>
	);
}

export const gameTagFilterOptions: FilterOption<GameTag>[] = [
	{ label: "Native VR", value: "VR" },
	{ label: "Demo", value: "Demo" },
];

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
