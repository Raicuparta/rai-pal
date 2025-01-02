import { Group, Stack } from "@mantine/core";
import { useCallback } from "react";
import { FilterMenu } from "@components/filters/filter-menu";
import { RefreshButton } from "@components/refresh-button";
import { AddGame } from "./add-game-button";
import { useAppEvent } from "@hooks/use-app-event";
import { events, GamesSortBy } from "@api/bindings";
import { useAtomValue, useSetAtom } from "jotai";
import { gameIdsAtom } from "@hooks/use-data";
import { selectedGameAtom } from "./games-state";
import { TableContainer } from "@components/table/table-container";
import { TableVirtuoso } from "react-virtuoso";
import { useVirtuosoHeaderContent } from "@hooks/use-virtuoso-header-content";
import { useVirtuosoTableComponents } from "@hooks/use-virtuoso-table-components";
import { GameRow, gameRowHeight } from "./game-row";
import { useDataQuery } from "@hooks/use-data-query";
import { gamesColumns } from "./games-columns";

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
		gamesColumns,
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
					// 2px for the bottom border in table.module.css
					fixedItemHeight={gameRowHeight + 2}
					overscan={50}
					increaseViewportBy={100}
					scrollSeekConfiguration={{
						enter: (velocity) => Math.abs(velocity) > 500,
						exit: (velocity) => Math.abs(velocity) < 400,
					}}
				/>
			</TableContainer>
		</Stack>
	);
}
