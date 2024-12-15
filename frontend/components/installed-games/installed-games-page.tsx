import { Group, Stack } from "@mantine/core";
import { useCallback } from "react";
import { FilterMenu } from "@components/filters/filter-menu";
import { RefreshButton } from "@components/refresh-button";
import { AddGame } from "./add-game-button";
import { useAppEvent } from "@hooks/use-app-event";
import {
	commands,
	events,
	InstalledGame,
	InstalledGameSortBy,
} from "@api/bindings";
import { useAtomValue, useSetAtom } from "jotai";
import { gameIdsAtom } from "@hooks/use-data";
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

export function InstalledGamesPage() {
	const gameIds = useAtomValue(gameIdsAtom);
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
					totalCount={gameIds.length}
					data={gameIds}
					defaultItemHeight={33}
				/>
			</TableContainer>
		</Stack>
	);
}
