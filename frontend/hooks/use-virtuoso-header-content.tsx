import { TableColumn, TableHead } from "@components/table/table-head";
import { useCallback } from "react";
import { InstalledGameSortBy } from "@api/bindings";

export function useVirtuosoHeaderContent<TKey extends string, TItem>(
	columns: TableColumn<TKey, TItem>[],
	onChangeSort?: (sort: InstalledGameSortBy) => void,
	sort?: InstalledGameSortBy,
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
