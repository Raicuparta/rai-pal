import { TableColumn, TableHead } from "@components/table/table-head";
import { useCallback } from "react";
import { TableSort } from "./use-table-sort";

export function useVirtuosoHeaderContent<TKey extends string>(
	columns: TableColumn<TKey, unknown>[],
	onChangeSort?: (sort: string) => void,
	sort?: TableSort,
) {
	return useCallback(
		() => (
			<TableHead
				columns={columns}
				onChangeSort={onChangeSort}
				sort={sort}
			/>
		),
		[columns, sort, onChangeSort],
	);
}
