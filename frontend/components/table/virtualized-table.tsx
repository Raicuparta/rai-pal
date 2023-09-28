import { useCallback, useMemo } from "react";
import { TableVirtuoso, TableVirtuosoProps } from "react-virtuoso";
import { TableHead, TableHeader } from "./table-head";
import { TableSort } from "@hooks/use-table-sort";
import { getTableComponents } from "./table-components";
import { TableContainer } from "./table-container";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
interface Props<TItem, TKey extends keyof TItem, Context = any>
	extends TableVirtuosoProps<TItem, Context> {
	readonly headerItems: TableHeader<TItem, TKey>[];
	readonly onChangeSort?: (sort: TKey) => void;
	readonly sort?: TableSort<TItem, TKey>;
	readonly onClickItem: (item: TItem) => void;
}

export function VirtualizedTable<
	TItem,
	TKey extends keyof TItem,
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	Context = any,
>({
	headerItems,
	sort,
	onChangeSort,
	onClickItem,
	...props
}: Props<TItem, TKey, Context>) {
	const renderHeaders = useCallback(
		() => (
			<TableHead
				headers={headerItems}
				onChangeSort={onChangeSort}
				sort={sort}
			/>
		),
		[headerItems, sort, onChangeSort],
	);

	const tableComponents = useMemo(
		() => getTableComponents(onClickItem),
		[onClickItem],
	);

	return (
		<TableContainer>
			<TableVirtuoso
				components={tableComponents}
				fixedHeaderContent={renderHeaders}
				{...props}
			/>
		</TableContainer>
	);
}
