import { InstalledGameRow } from "@components/installed-games/installed-game-row";
import { Table } from "@mantine/core";
import React, { useMemo } from "react";
import { TableComponents } from "react-virtuoso";

export function useVirtuosoTableComponents<
	TItem,
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	TContext = any,
>(): TableComponents<TItem, TContext> {
	return useMemo(
		() => ({
			// eslint-disable-next-line react/display-name
			TableBody: React.forwardRef((props, ref) => (
				<Table.Tbody
					{...props}
					ref={ref}
				/>
			)),
			Table: (props) => (
				<Table
					{...props}
					highlightOnHover
				/>
			),
			// eslint-disable-next-line react/display-name
			TableHead: React.forwardRef((props, ref) => (
				<Table.Thead
					{...props}
					ref={ref}
				/>
			)),
			TableRow: InstalledGameRow,
		}),
		[],
	);
}
