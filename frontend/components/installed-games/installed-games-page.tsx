import { Group, Stack } from "@mantine/core";
import { useCallback } from "react";
import { FilterMenu } from "@components/filters/filter-menu";
import { RefreshButton } from "@components/refresh-button";
import { AddGame } from "./add-game-button";
import { useAppEvent } from "@hooks/use-app-event";
import { commands, events, GamesSortBy } from "@api/bindings";
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

export function InstalledGamesPage() {
	const gameIds = useAtomValue(gameIdsAtom);
	const setSelectedGame = useSetAtom(selectedInstalledGameAtom);
	const [dataQuery, setDataQuery] = useDataQuery(
		commands.setGamesQuery,
		commands.getGamesQuery,
	);

	useAppEvent(events.selectInstalledGame, ([providerId, gameId]) => {
		setSelectedGame({
			providerId,
			gameId,
		});
	});

	const columns = useVisibleInstalledGameColumns();

	const onChangeSort = useCallback(
		(sortBy: GamesSortBy) => {
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
					setterCommand={commands.setGamesQuery}
					getterCommand={commands.getGamesQuery}
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
					overscan={200}
					increaseViewportBy={200}
				/>
			</TableContainer>
		</Stack>
	);
}
