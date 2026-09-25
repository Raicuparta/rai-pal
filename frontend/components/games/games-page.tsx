import { Card, Group, Stack } from "@mantine/core";
import { GamesTable } from "./games-table";
import { GamePage } from "./game-page";
import { useSelectedGame } from "@hooks/use-selected-game";
import { GamesSidebar } from "./games-sidebar";

export function GamesPage() {
	const { selectedGame, gameMods } = useSelectedGame();
	const showGamePage = Boolean(selectedGame && gameMods);

	return (
		<Stack h="100%">
			{showGamePage && selectedGame && gameMods && (
				<GamePage
					game={selectedGame}
					mods={gameMods}
				/>
			)}
			<Card
				p={0}
				flex={1}
				display={showGamePage ? "none" : undefined}
				bg="dark"
			>
				<Group
					flex={1}
					mih={0}
					wrap="nowrap"
					align="stretch"
					gap={0}
				>
					<GamesSidebar />
					<GamesTable />
				</Group>
			</Card>
		</Stack>
	);
}
