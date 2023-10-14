import { TableColumn } from "@components/table/table-head";
import React, { useCallback } from "react";

export function useTableRowContent<TItem>(columns: TableColumn<TItem>[]) {
	const TableRowContentInner = useCallback(
		(_: number, item: TItem) => (
			<>
				{columns.map((column) => (
					<React.Fragment key={column.id}>
						{column.renderCell ? column.renderCell(item) : "TODO"}
					</React.Fragment>
				))}
			</>
		),
		[columns],
	);

	return TableRowContentInner;
}
