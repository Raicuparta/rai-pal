import { Group, Stack } from "@mantine/core";
import { FilterMenu } from "@components/filters/filter-menu";
import { RefreshButton } from "@components/refresh-button";
import { AddGame } from "./add-game-button";
import { useAppEvent } from "@hooks/use-app-event";
import { useAtom } from "jotai";
import { selectedGameAtom } from "./games-state";
import { GamesTable } from "./games-table";
import { GameModal } from "./game-modal";

export function GamesPage() {
	const [selectedGame, setSelectedGame] = useAtom(selectedGameAtom);

	useAppEvent("selectInstalledGame", "games-page", ([providerId, gameId]) => {
		setSelectedGame({ providerId, gameId });
	});

	return (
		<Stack h="100%">
			{selectedGame && (
				<GameModal
					providerId={selectedGame.providerId}
					gameId={selectedGame.gameId}
				/>
			)}
			<Group>
				<AddGame />
				<FilterMenu />
				<RefreshButton />
			</Group>
			<GamesTable />
		</Stack>
	);
}
