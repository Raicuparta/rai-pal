import { Game } from "@api/bindings";
import { GameRowInner } from "@components/games/game-row";
import { Table } from "@mantine/core";
import React, { useMemo } from "react";
import { TableComponents } from "react-virtuoso";

const defaultGame: Game = {
	id: "",
	providerId: "Manual",
	installedGame: null,
	ownedGame: null,
	remoteGame: null,
	fromSubscriptions: [],
	providerCommands: {},
	releaseDate: null,
	tags: [],
	thumbnailUrl: null,
	title: {
		display: "...",
		normalized: ["..."],
	},
} as unknown as Game;

export function useVirtuosoTableComponents<TItem>(
	rowComponent: TableComponents<TItem, unknown>["TableRow"],
): TableComponents<TItem, unknown> {
	return useMemo(
		() => ({
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
			TableRow: rowComponent,
			ScrollSeekPlaceholder: () => <GameRowInner game={defaultGame} />,
		}),
		[rowComponent],
	);
}
