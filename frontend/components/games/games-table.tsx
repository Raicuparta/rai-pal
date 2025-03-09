import { useEffect, useRef } from "react";
import { GameId, GamesSortBy } from "@api/bindings";
import { useAtomValue } from "jotai";
import { gameDataAtom } from "@hooks/use-data";
import { TableContainer } from "@components/table/table-container";
import {
	TableComponents,
	TableVirtuoso,
	TableVirtuosoHandle,
} from "react-virtuoso";
import { GameRow, gameRowHeight } from "./game-row";
import { useDataQuery } from "@hooks/use-data-query";
import { gamesColumns } from "./games-columns";
import styles from "./games.module.css";
import { Table } from "@mantine/core";
import React from "react";
import { TableHead } from "@components/table/table-head";

const tableComponents: TableComponents<GameId, unknown> = {
	TableBody: React.forwardRef(function TableBody(props, ref) {
		return (
			<Table.Tbody
				{...props}
				ref={ref}
			/>
		);
	}),
	Table: (props) => (
		<Table
			{...props}
			highlightOnHover
		/>
	),
	TableHead: React.forwardRef(function TableHead(props, ref) {
		return (
			<Table.Thead
				{...props}
				ref={ref}
			/>
		);
	}),
	TableRow: GameRow,
};

export function GamesTable() {
	const gameData = useAtomValue(gameDataAtom);
	const [dataQuery, setDataQuery] = useDataQuery();
	const tableRef = useRef<TableVirtuosoHandle>(null);

	const onChangeSort = (sortBy: GamesSortBy) => {
		const sortDescending =
			sortBy === dataQuery?.sortBy && !dataQuery?.sortDescending;

		setDataQuery({
			sortBy,
			sortDescending,
		});
	};

	const fixedHeaderContent = () => (
		<TableHead
			columns={gamesColumns}
			onChangeSort={onChangeSort}
			sortBy={dataQuery?.sortBy}
			sortDescending={dataQuery?.sortDescending}
		/>
	);

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
				fixedHeaderContent={fixedHeaderContent}
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
