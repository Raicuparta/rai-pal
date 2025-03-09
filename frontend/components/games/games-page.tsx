import { Group, Stack } from "@mantine/core";
import { FilterMenu } from "@components/filters/filter-menu";
import { RefreshButton } from "@components/refresh-button";
import { AddGame } from "./add-game-button";
import { useAppEvent } from "@hooks/use-app-event";
import { useSetAtom } from "jotai";
import { selectedGameAtom } from "./games-state";
import { GamesTable } from "./games-table";

export function GamesPage() {
	const setSelectedGame = useSetAtom(selectedGameAtom);

	useAppEvent("selectInstalledGame", "games-page", ([providerId, gameId]) => {
		setSelectedGame({
			providerId,
			gameId,
		});
	});

	return (
		<Stack h="100%">
			<Group>
				<AddGame />
				<FilterMenu />
				<RefreshButton />
			</Group>
			<GamesTable />
		</Stack>
	);
}
