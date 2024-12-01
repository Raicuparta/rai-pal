import { Group, Stack } from "@mantine/core";
import { useCallback, useMemo } from "react";
import { FilterMenu } from "@components/filters/filter-menu";
import { RefreshButton } from "@components/refresh-button";
import { OwnedGameColumnsId, ownedGamesColumns } from "./owned-games-columns";
import { usePersistedState } from "@hooks/use-persisted-state";
import { useAppEvent } from "@hooks/use-app-event";
import { commands, events, InstalledGame, ProviderId } from "@api/bindings";
import { useAtomValue, useSetAtom } from "jotai";
import { providerDataAtom } from "@hooks/use-data";
import { TableContainer } from "@components/table/table-container";
import { TableVirtuoso } from "react-virtuoso";
import { useVirtuosoHeaderContent } from "@hooks/use-virtuoso-header-content";
import { useVirtuosoTableComponents } from "@hooks/use-virtuoso-table-components";
import { selectedOwnedGameAtom } from "./selected-owned-game";
import { OwnedGameRow } from "./owned-game-row";
import { FixOwnedGamesButton } from "./fix-owned-games-button";

export type TableSortMethod = (
	gameA: InstalledGame,
	gameB: InstalledGame,
) => number;

const defaultColumns: OwnedGameColumnsId[] = [
	"thumbnail",
	"engine",
	"provider",
];

export type OwnedGameId = {
	readonly provider: ProviderId;
	readonly id: string;
};

export function OwnedGamesPage() {
	const providerData = useAtomValue(providerDataAtom);
	const setSelectedGame = useSetAtom(selectedOwnedGameAtom);

	useAppEvent(events.selectInstalledGame, ([provider, id]) => {
		setSelectedGame({
			provider,
			id,
		});
	});

	const ownedGames = useMemo(() => {
		const result: OwnedGameId[] = [];
		for (const providerId of Object.keys(providerData) as ProviderId[]) {
			const ownedGameIds = providerData[providerId]?.ownedGames;
			if (!ownedGameIds) continue;

			for (const ownedGameId of ownedGameIds) {
				result.push({
					id: ownedGameId,
					provider: providerId,
				});
			}
		}

		return result;
	}, [providerData]);

	const [visibleColumnIds, setVisibleColumnIds] = usePersistedState<
		OwnedGameColumnsId[]
	>(defaultColumns, "owned-visible-columns");

	const filteredColumns = useMemo(
		() =>
			ownedGamesColumns.filter(
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

	const tableComponents = useVirtuosoTableComponents(OwnedGameRow);

	return (
		<Stack h="100%">
			<Group>
				<FixOwnedGamesButton />
				<FilterMenu
					setterCommand={commands.setInstalledGamesFilter} // TODO owned
					getterCommand={commands.getInstalledGamesFilter} // TODO owned
				/>
				<RefreshButton />
			</Group>
			<TableContainer>
				<TableVirtuoso
					style={{ overflowY: "scroll" }}
					components={tableComponents}
					fixedHeaderContent={renderHeaders}
					totalCount={ownedGames.length}
					data={ownedGames}
					defaultItemHeight={33}
				/>
			</TableContainer>
		</Stack>
	);
}
