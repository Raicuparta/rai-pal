import { Group, Stack } from "@mantine/core";
import { useCallback } from "react";
import { FilterMenu } from "@components/filters/filter-menu";
import { RefreshButton } from "@components/refresh-button";
import { AddGame } from "./add-game-button";
import { useAppEvent } from "@hooks/use-app-event";
import { events, GamesSortBy } from "@api/bindings";
import { useAtomValue, useSetAtom } from "jotai";
import { gameIdsAtom } from "@hooks/use-data";
import { selectedGameAtom, useVisibleGamesColumns } from "./games-state";
import { TableContainer } from "@components/table/table-container";
import { TableVirtuoso } from "react-virtuoso";
import { useVirtuosoHeaderContent } from "@hooks/use-virtuoso-header-content";
import { useVirtuosoTableComponents } from "@hooks/use-virtuoso-table-components";
import { GameRow } from "./game-row";
import { useDataQuery } from "@hooks/use-data-query";

export function GamesPage() {
	const gameIds = useAtomValue(gameIdsAtom);
	const setSelectedGame = useSetAtom(selectedGameAtom);
	const [dataQuery, setDataQuery] = useDataQuery();

	useAppEvent(events.selectInstalledGame, ([providerId, gameId]) => {
		setSelectedGame({
			providerId,
			gameId,
		});
	});

	const columns = useVisibleGamesColumns();

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

	const tableComponents = useVirtuosoTableComponents(GameRow);

	return (
		<Stack h="100%">
			<Group>
				<AddGame />
				<FilterMenu />
				<RefreshButton />
			</Group>
			<TableContainer>
				<TableVirtuoso
					style={{ overflowY: "scroll" }}
					components={tableComponents}
					fixedHeaderContent={renderHeaders}
					totalCount={gameIds.length}
					data={gameIds}
					defaultItemHeight={75}
					increaseViewportBy={200}
				/>
			</TableContainer>
		</Stack>
	);
}
