import { Card, Group, Stack } from "@mantine/core";
import { FilterMenu } from "@components/filters/filter-menu";
import { RefreshButton } from "@components/refresh-button";
import { AddGame } from "./add-game-button";
import { useAppEvent } from "@hooks/use-app-event";
import { useAtom } from "jotai";
import { selectedGameAtom } from "./games-state";
import { GamesTable } from "./games-table";
import { GameModal } from "./game-modal";
import { useGame } from "@hooks/use-game";

export function GamesPage() {
	const [selectedGame, setSelectedGame] = useAtom(selectedGameAtom);

	useAppEvent("selectGame", "games-page", ([providerId, gameId]) => {
		setSelectedGame({ providerId, gameId });
	});

	const game = useGame(selectedGame?.providerId, selectedGame?.gameId);

	return (
		<Stack h="100%">
			{game && <GameModal game={game} />}
			{!game && (
				<Group>
					<AddGame />
					<FilterMenu />
					<RefreshButton />
				</Group>
			)}
			<Card
				p={0}
				flex={1}
				display={game ? "none" : undefined}
				bg="dark"
			>
				<GamesTable />
			</Card>
		</Stack>
	);
}
