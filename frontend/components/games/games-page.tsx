import { Card, Group, Stack } from "@mantine/core";
import { GamesTable } from "./games-table";
import { GameModal } from "./game-modal";
import { useSelectedGame } from "@hooks/use-selected-game";
import { GamesSidebar } from "./games-sidebar";

export function GamesPage() {
	const { selectedGame, gameMods } = useSelectedGame();
	const showModal = Boolean(selectedGame && gameMods);

	return (
		<Stack h="100%">
			{showModal && selectedGame && gameMods && (
				<GameModal
					game={selectedGame}
					mods={gameMods}
				/>
			)}
			<Card
				p={0}
				flex={1}
				display={showModal ? "none" : undefined}
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
