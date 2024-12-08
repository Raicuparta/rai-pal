import { Group, Stack } from "@mantine/core";
import { useCallback, useMemo } from "react";
import { FilterMenu } from "@components/filters/filter-menu";
import { RefreshButton } from "@components/refresh-button";
import { AddGame } from "./add-game-button";
import { useAppEvent } from "@hooks/use-app-event";
import {
	commands,
	events,
	InstalledGame,
	InstalledGameSortBy,
	ProviderId,
} from "@api/bindings";
import { useAtomValue, useSetAtom } from "jotai";
import { providerDataAtom } from "@hooks/use-data";
import {
	selectedInstalledGameAtom,
	useVisibleInstalledGameColumns,
} from "./installed-games-state";
import { TableContainer } from "@components/table/table-container";
import { TableVirtuoso } from "react-virtuoso";
import { useVirtuosoHeaderContent } from "@hooks/use-virtuoso-header-content";
import { useVirtuosoTableComponents } from "@hooks/use-virtuoso-table-components";
import { InstalledGameRow } from "./installed-game-row";
import { useDataQuery } from "@hooks/use-data-query";

export type TableSortMethod = (
	gameA: InstalledGame,
	gameB: InstalledGame,
) => number;

export type InstalledGameId = {
	readonly provider: ProviderId;
	readonly id: string;
};

export function InstalledGamesPage() {
	const providerData = useAtomValue(providerDataAtom);
	const setSelectedGame = useSetAtom(selectedInstalledGameAtom);
	const [dataQuery, setDataQuery] = useDataQuery(
		commands.setInstalledGamesFilter,
		commands.getInstalledGamesFilter,
	);

	useAppEvent(events.selectInstalledGame, ([provider, id]) => {
		setSelectedGame({
			provider,
			id,
		});
	});

	const installedGames = useMemo(() => {
		const result: InstalledGameId[] = [];
		const installedGameIds = providerData.installedGames;

		for (const installedGameId of installedGameIds) {
			result.push({
				id: installedGameId.gameId,
				provider: installedGameId.providerId,
			});
		}

		return result;
	}, [providerData]);

	const columns = useVisibleInstalledGameColumns();

	const onChangeSort = useCallback(
		(sortBy: InstalledGameSortBy) => {
			const sortDescending =
				sortBy === dataQuery?.sortBy && !dataQuery?.sortDescending;

			setDataQuery({
				sortBy,
				sortDescending,
			});
		},
		[dataQuery, setDataQuery],
	);

	const renderHeaders = useVirtuosoHeaderContent(
		columns,
		onChangeSort,
		dataQuery?.sortBy,
		dataQuery?.sortDescending,
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
