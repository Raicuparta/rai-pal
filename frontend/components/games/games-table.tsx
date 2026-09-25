import { useEffect, useRef } from "react";
import { GameProviderId } from "@api/bindings";
import { useAtomValue } from "jotai";
import { gameDataAtom, loadingTasksAtom } from "@hooks/use-data";
import { TableContainer } from "@components/table/table-container";
import {
	TableComponents,
	TableVirtuoso,
	TableVirtuosoHandle,
} from "react-virtuoso";
import { GameRow, gameRowHeight } from "./game-row";
import { useDataQuery } from "@hooks/use-data-query";
import { GamesColgroup } from "./games-columns";
import { Alert, Table } from "@mantine/core";
import React from "react";
import { useLocalization } from "@hooks/use-localization";
import { usePageScrollElement } from "@components/page";

const tableComponents: TableComponents<[GameProviderId, string], unknown> = {
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
		>
			<GamesColgroup />
			{props.children}
		</Table>
	),
	TableRow: GameRow,
};

export function GamesTable() {
	const gameData = useAtomValue(gameDataAtom);
	const loading = useAtomValue(loadingTasksAtom);
	const [dataQuery] = useDataQuery();
	const tableRef = useRef<TableVirtuosoHandle>(null);
	const pageScrollElement = usePageScrollElement();
	const { t } = useLocalization("gamesPage");

	useEffect(() => {
		if (tableRef.current) {
			tableRef.current.scrollToIndex(0);
		}
	}, [dataQuery]);

	if (gameData.totalCount === 0 && loading.length > 0) {
		return <Alert flex={1}>{t("emptyGamesLoading")}</Alert>;
	}

	if (gameData.totalCount === 0) {
		return <Alert flex={1}>{t("emptyGamesList")}</Alert>;
	}

	if (gameData.gameIds.length === 0) {
		return <Alert flex={1}>{t("emptyFilteredGamesList")}</Alert>;
	}

	return (
		<TableContainer>
			<TableVirtuoso
				ref={tableRef}
				customScrollParent={pageScrollElement ?? undefined}
				components={tableComponents}
				data={gameData.gameIds}
				fixedItemHeight={gameRowHeight}
				overscan={50}
				increaseViewportBy={100}
				computeItemKey={(index) =>
					`${gameData.gameIds[index]?.[0]}${gameData.gameIds[index]?.[1]}`
				}
			/>
		</TableContainer>
	);
}
