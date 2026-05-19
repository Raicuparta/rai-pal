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
import { useGame } from "@hooks/use-game";
import { useLocalization } from "@hooks/use-localization";

export function GamesPage() {
	const t = useLocalization("gamesPage");
	const [selectedGame, setSelectedGame] = useAtom(selectedGameAtom);

	useAppEvent("selectGame", "games-page", ([providerId, gameId]) => {
		setSelectedGame({ providerId, gameId });
	});

	const game = useGame(selectedGame?.providerId, selectedGame?.gameId);

	return (
		<Stack h="100%">
			{game && (
				<>
					<Group>
						<Button
							onClick={() => setSelectedGame(null)}
							leftSection={<IconArrowLeft />}
						>
							{t("backToGamesList")}
						</Button>
					</Group>

					<GameModal game={game} />
				</>
			)}
			{!game && (
				<Group>
					<AddGame />
					<FilterMenu />
					<RefreshButton />
				</Group>
			)}
			<Stack
				flex={1}
				display={game ? "none" : undefined}
			>
				<GamesTable />
			</Stack>
		</Stack>
	);
}
