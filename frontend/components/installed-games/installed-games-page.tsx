import { Group, Stack } from "@mantine/core";
import { useCallback, useMemo } from "react";
import { FilterMenu } from "@components/filters/filter-menu";
import { RefreshButton } from "@components/refresh-button";
import {
	InstalledGameColumnsId,
	installedGamesColumns,
} from "./installed-games-columns";
import { usePersistedState } from "@hooks/use-persisted-state";
import { AddGame } from "./add-game-button";
import { useAppEvent } from "@hooks/use-app-event";
import { commands, events, InstalledGame, ProviderId } from "@api/bindings";
import { useAtomValue, useSetAtom } from "jotai";
import { providerDataAtom } from "@hooks/use-data";
import { selectedInstalledGameAtom } from "./selected-installed-game";
import { TableContainer } from "@components/table/table-container";
import { TableVirtuoso } from "react-virtuoso";
import { useVirtuosoHeaderContent } from "@hooks/use-virtuoso-header-content";
import { useVirtuosoTableComponents } from "@hooks/use-virtuoso-table-components";
import { InstalledGameRow } from "./installed-game-row";

export type TableSortMethod = (
	gameA: InstalledGame,
	gameB: InstalledGame,
) => number;

const defaultColumns: InstalledGameColumnsId[] = [
	"thumbnail",
	"engine",
	"provider",
];

export type InstalledGameId = {
	readonly provider: ProviderId;
	readonly id: string;
};
export function InstalledGamesPage() {
	const providerData = useAtomValue(providerDataAtom);
	const setSelectedGame = useSetAtom(selectedInstalledGameAtom);

	useAppEvent(events.selectInstalledGame, ([provider, id]) => {
		setSelectedGame({
			provider,
			id,
		});
	});

	const installedGames = useMemo(() => {
		const result: InstalledGameId[] = [];
		for (const providerId of Object.keys(providerData) as ProviderId[]) {
			const installedGameIds = providerData[providerId]?.installedGames;
			if (!installedGameIds) continue;

			for (const installedGameId of installedGameIds) {
				result.push({
					id: installedGameId,
					provider: providerId,
				});
			}
		}

		return result;
	}, [providerData]);

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

	const onChangeSort = useCallback(() => {
		console.log("Not implemented");
	}, []);

	const sort = undefined;

	const renderHeaders = useVirtuosoHeaderContent(
		filteredColumns,
		onChangeSort,
		sort,
	);

	const tableComponents = useVirtuosoTableComponents(InstalledGameRow);

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
			<TableContainer>
				<TableVirtuoso
					style={{ overflowY: "scroll" }}
					components={tableComponents}
					fixedHeaderContent={renderHeaders}
					totalCount={installedGames.length}
					data={installedGames}
					defaultItemHeight={33}
				/>
			</TableContainer>
		</Stack>
	);
}
