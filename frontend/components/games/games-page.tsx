import { Group, Stack } from "@mantine/core";
import { useCallback, useEffect, useRef } from "react";
import { FilterMenu } from "@components/filters/filter-menu";
import { RefreshButton } from "@components/refresh-button";
import { AddGame } from "./add-game-button";
import { useAppEvent } from "@hooks/use-app-event";
import { GamesSortBy } from "@api/bindings";
import { useAtomValue, useSetAtom } from "jotai";
import { gameDataAtom } from "@hooks/use-data";
import { selectedGameAtom } from "./games-state";
import { TableContainer } from "@components/table/table-container";
import { TableVirtuoso, TableVirtuosoHandle } from "react-virtuoso";
import { useVirtuosoHeaderContent } from "@hooks/use-virtuoso-header-content";
import { useVirtuosoTableComponents } from "@hooks/use-virtuoso-table-components";
import { GameRow, gameRowHeight } from "./game-row";
import { useDataQuery } from "@hooks/use-data-query";
import { gamesColumns } from "./games-columns";
import styles from "./games.module.css";

export function GamesPage() {
	const gameData = useAtomValue(gameDataAtom);
	const setSelectedGame = useSetAtom(selectedGameAtom);
	const [dataQuery, setDataQuery] = useDataQuery();
	const tableRef = useRef<TableVirtuosoHandle>(null);

	useAppEvent("selectInstalledGame", ([providerId, gameId]) => {
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

	useEffect(() => {
		if (tableRef.current) {
			tableRef.current.scrollToIndex(0);
		}
	}, [dataQuery]);

	return (
		<Stack h="100%">
			<Group>
				<AddGame />
				<FilterMenu />
				<RefreshButton />
			</Group>
			<TableContainer>
				<TableVirtuoso
					ref={tableRef}
					className={styles.table}
					style={{ overflowY: "scroll" }}
					components={tableComponents}
					fixedHeaderContent={renderHeaders}
					totalCount={gameData.gameIds.length}
					data={gameData.gameIds}
					fixedItemHeight={gameRowHeight}
					overscan={50}
					increaseViewportBy={100}
				/>
			</TableContainer>
		</Stack>
	);
}
