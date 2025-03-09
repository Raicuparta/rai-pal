import { useCallback, useEffect, useRef } from "react";
import { GamesSortBy } from "@api/bindings";
import { useAtomValue } from "jotai";
import { gameDataAtom } from "@hooks/use-data";
import { TableContainer } from "@components/table/table-container";
import { TableVirtuoso, TableVirtuosoHandle } from "react-virtuoso";
import { useVirtuosoHeaderContent } from "@hooks/use-virtuoso-header-content";
import { useVirtuosoTableComponents } from "@hooks/use-virtuoso-table-components";
import { GameRow, gameRowHeight } from "./game-row";
import { useDataQuery } from "@hooks/use-data-query";
import { gamesColumns } from "./games-columns";
import styles from "./games.module.css";

export function GamesTable() {
	const gameData = useAtomValue(gameDataAtom);
	const [dataQuery, setDataQuery] = useDataQuery();
	const tableRef = useRef<TableVirtuosoHandle>(null);

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
		<TableContainer>
			<TableVirtuoso
				ref={tableRef}
				className={styles.table}
				components={tableComponents}
				fixedHeaderContent={renderHeaders}
				data={gameData.gameIds}
				fixedItemHeight={gameRowHeight}
				overscan={50}
				increaseViewportBy={100}
				computeItemKey={(index) =>
					`${gameData.gameIds[index]?.providerId}${gameData.gameIds[index]?.gameId}`
				}
			/>
		</TableContainer>
	);
}
