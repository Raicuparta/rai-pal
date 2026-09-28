import { Stack } from "@mantine/core";
import { GamesTable } from "./games-table";
import { GamePage } from "./game-page";
import { useSelectedGame } from "@hooks/use-selected-game";
import { GamesSidebar } from "./games-sidebar";
import { Page } from "@components/page";

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
			<Page
				display={showGamePage ? "none" : undefined}
				scrollable={false}
				sidebar={<GamesSidebar />}
			>
				<GamesTable />
			</Page>
		</Stack>
	);
}
