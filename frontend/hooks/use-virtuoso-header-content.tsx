import { TableColumn, TableHead } from "@components/table/table-head";
import { GamesSortBy } from "@api/bindings";

export function useVirtuosoHeaderContent<TKey extends string, TItem, TSort>(
	columns: TableColumn<TKey, TItem, TSort>[],
	onChangeSort?: (sort: TSort) => void,
	sort?: GamesSortBy,
	sortDescending?: boolean,
) {
	const VirtuosoHeaderContent = () => (
		<TableHead
			columns={columns}
			onChangeSort={onChangeSort}
			sortBy={sort}
			sortDescending={sortDescending}
		/>
	);

	return VirtuosoHeaderContent;
}
