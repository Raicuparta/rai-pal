import { GameTag, OwnedGame } from "@api/bindings";
import { Badge, Box, Flex, Stack, Table } from "@mantine/core";
import { FilterOption } from "./table/table-head";

function getTagDisplayName(tag: GameTag) {
	return (
		gameTagFilterOptions.find((option) => option.value === tag)?.label || tag
	);
}

export function renderGameTagsCell(ownedGame: OwnedGame | undefined) {
	return (
		<Table.Td p={0}>
			<Stack
				gap={0}
				align="center"
			>
				{ownedGame?.tags.sort().map((tag) => (
					<Badge
						key={tag}
						color="gray"
						size="xs"
						fw={1}
					>
						{getTagDisplayName(tag)}
					</Badge>
				))}
			</Stack>
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

export function getGameTagsSortValue(ownedGame: OwnedGame | undefined) {
	return ownedGame
		? `${ownedGame.tags.length}${ownedGame.tags.sort().join(",")}`
		: "";
}
