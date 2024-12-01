import { GameTag, OwnedGame } from "@api/bindings";
import { Table } from "@mantine/core";
import styles from "./game-tags.module.css";

export function renderGameTagsCell(ownedGame: OwnedGame | null) {
	return (
		<Table.Td p={0}>
			<div className={styles.wrapper}>
				{ownedGame?.tags.sort().map((tag) => (
					<span
						className={styles.tag}
						key={tag}
					>
						{tag}
					</span>
				))}
			</div>
		</Table.Td>
	);
}

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
