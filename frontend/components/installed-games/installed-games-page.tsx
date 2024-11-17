import { Group, Stack } from "@mantine/core";
import { useMemo, useState } from "react";
import { InstalledGameModal } from "./installed-game-modal";
import { FilterMenu } from "@components/filters/filter-menu";
import { VirtualizedTable } from "@components/table/virtualized-table";
import { RefreshButton } from "@components/refresh-button";
import {
	InstalledGameColumnsId,
	installedGamesColumns,
} from "./installed-games-columns";
import { usePersistedState } from "@hooks/use-persisted-state";
import { AddGame } from "./add-game-button";
import { ProcessedInstalledGame } from "@hooks/use-processed-installed-games";
import { useAppEvent } from "@hooks/use-app-event";
import { commands, events, ProviderId } from "@api/bindings";
import { useAtomValue } from "jotai";
import { providerDataAtom } from "@hooks/use-data";
import { InstalledGameRow } from "./installed-game-row";

export type TableSortMethod = (
	gameA: ProcessedInstalledGame,
	gameB: ProcessedInstalledGame,
) => number;

const defaultColumns: InstalledGameColumnsId[] = [
	"thumbnail",
	"engine",
	"provider",
];

export function InstalledGamesPage() {
	const providerData = useAtomValue(providerDataAtom);

	const [selectedGameId, setSelectedGameId] = useState<string>();

	useAppEvent(events.selectInstalledGame, (gameId: string) => {
		setSelectedGameId(gameId);
	});

	const installedGames = useMemo(() => {
		const result: [ProviderId, string][] = [];
		for (const providerId of Object.keys(providerData) as ProviderId[]) {
			const installedGames = providerData[providerId]?.installedGames;
			if (!installedGames) continue;

			for (const installedGame of installedGames) {
				result.push([providerId, installedGame]);
			}
		}

		return result;
	}, [providerData]);

	const selectedGame = useMemo(
		() =>
			installedGames && selectedGameId
				? installedGames[selectedGameId]
				: undefined,
		[installedGames, selectedGameId],
	);

	const [visibleColumnIds, setVisibleColumnIds] = usePersistedState<
		InstalledGameColumnsId[]
	>(defaultColumns, "installed-visible-columns");

	const filteredColumns = useMemo(
		() =>
			installedGamesColumns.filter(
				(column) => !column.hidable || visibleColumnIds.includes(column.id),
			),
		[visibleColumnIds],
	);

	return (
		<Stack h="100%">
			<Group>
				<AddGame />
				<FilterMenu
					setterCommand={commands.setInstalledGamesFilter}
					getterCommand={commands.getInstalledGamesFilter}
				/>
				<RefreshButton />
			</Group>
			{selectedGame ? (
				<InstalledGameModal
					game={selectedGame}
					onClose={() => setSelectedGameId(undefined)}
				/>
			) : null}
			<VirtualizedTable
				// data={installedGames}
				totalCount={installedGames.length}
				itemContent={(index) => (
					<InstalledGameRow
						provider_id={installedGames[index][0]}
						game_id={installedGames[index][1]}
					/>
				)}
				columns={filteredColumns}
				// onChangeSort={setSort}
				// onClickItem={(game) => setSelectedGameId(game.id)}
				// sort={sort}
			/>
		</Stack>
	);
}
