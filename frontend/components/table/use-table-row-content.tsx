import { TableColumn } from "@components/table/table-head";
import React, { useCallback } from "react";

export function useTableRowContent<TKey extends string, TItem>(
	columns: TableColumn<TKey, TItem>[],
) {
	const TableRowContentInner = useCallback(
		(_: number, item: TItem) => (
			<>
				{columns.map((column) => (
					<React.Fragment key={column.id}>
						{column.renderCell(item)}
					</React.Fragment>
				))}
			</>
		),
		[columns],
	);

	return TableRowContentInner;
}
