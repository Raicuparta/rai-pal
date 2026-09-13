import { Card, Group, Stack } from "@mantine/core";
import { FilterMenu } from "@components/filters/filter-menu";
import { RefreshButton } from "@components/refresh-button";
import { AddGame } from "./add-game-button";
import { GamesTable } from "./games-table";
import { GameModal } from "./game-modal";
import { useSelectedGame } from "@hooks/use-selected-game";

export function GamesPage() {
	const { selectedGame, gameMods } = useSelectedGame();

	return (
		<Stack h="100%">
			{selectedGame && gameMods ? (
				<GameModal
					game={selectedGame}
					mods={gameMods}
				/>
			) : (
				<Group>
					<AddGame />
					<FilterMenu />
					<RefreshButton />
				</Group>
			)}
			<Card
				p={0}
				flex={1}
				display={selectedGame && gameMods ? "none" : undefined}
				bg="dark"
			>
				<GamesTable />
			</Card>
		</Stack>
	);
}
