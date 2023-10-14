import { TableColumn } from "@components/table/table-head";
import React, { useCallback } from "react";

export function useTableRowContent<TItem>(headers: TableColumn<TItem>[]) {
	const TableRowContentInner = useCallback(
		(_: number, item: TItem) => (
			<>
				{headers.map((header) => (
					<React.Fragment key={header.id}>
						{header.renderCell ? header.renderCell(item) : "TODO"}
					</React.Fragment>
				))}
			</>
		),
		[headers],
	);

	return TableRowContentInner;
}
