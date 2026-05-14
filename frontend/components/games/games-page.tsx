import { Button, Group, Stack } from "@mantine/core";
import { FilterMenu } from "@components/filters/filter-menu";
import { RefreshButton } from "@components/refresh-button";
import { AddGame } from "./add-game-button";
import { useAppEvent } from "@hooks/use-app-event";
import { useAtom } from "jotai";
import { selectedGameAtom } from "./games-state";
import { GamesTable } from "./games-table";
import { GameModal } from "./game-modal";
import { IconArrowLeft } from "@tabler/icons-react";

export function GamesPage() {
	const [selectedGame, setSelectedGame] = useAtom(selectedGameAtom);

	useAppEvent("selectGame", "games-page", ([providerId, gameId]) => {
		setSelectedGame({ providerId, gameId });
	});

	return (
		<Stack h="100%">
			{selectedGame && (
				<Group>
					<Button
						onClick={() => setSelectedGame(null)}
						leftSection={<IconArrowLeft />}
					>
						Back to game list
					</Button>
				</Group>
			)}
			{!selectedGame && (
				<Group>
					<AddGame />
					<FilterMenu />
					<RefreshButton />
				</Group>
			)}
			{selectedGame && (
				<GameModal
					providerId={selectedGame.providerId}
					gameId={selectedGame.gameId}
				/>
			)}
			<Stack
				flex={1}
				hidden={Boolean(selectedGame)}
				display={selectedGame ? "none" : undefined}
			>
				<GamesTable />
			</Stack>
		</Stack>
	);
}
