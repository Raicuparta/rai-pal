import { Table } from "@mantine/core";
import React from "react";
import { TableComponents } from "react-virtuoso";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function getTableComponents<TItem, TContext = any>(
	onClickItem: (item: TItem) => void,
): TableComponents<TItem, TContext> {
	return {
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
		// eslint-disable-next-line react/display-name
		TableRow: (props) => (
			<Table.Tr
				onClick={() => onClickItem(props.item)}
				{...props}
			/>
		),
	};
}
