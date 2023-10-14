import { useCallback, useMemo } from "react";
import { TableVirtuoso, TableVirtuosoProps } from "react-virtuoso";
import { TableHead, TableColumn } from "./table-head";
import { TableSort } from "@hooks/use-table-sort";
import { getTableComponents } from "./table-components";
import { TableContainer } from "./table-container";
import { useTableRowContent } from "./use-table-row-content";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
interface Props<TItem, Context = any>
	extends TableVirtuosoProps<TItem, Context> {
	readonly columns: TableColumn<TItem>[];
	readonly onChangeSort?: (sort: string) => void;
	readonly sort?: TableSort;
	readonly onClickItem: (item: TItem) => void;
}

export function VirtualizedTable<
	TItem,
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	Context = any,
>({
	columns,
	sort,
	onChangeSort,
	onClickItem,
	...props
}: Props<TItem, Context>) {
	const renderHeaders = useCallback(
		() => (
			<TableHead
				headers={columns}
				onChangeSort={onChangeSort}
				sort={sort}
			/>
		),
		[columns, sort, onChangeSort],
	);

	const tableComponents = useMemo(
		() => getTableComponents(onClickItem),
		[onClickItem],
	);

	const itemContent = useTableRowContent(columns);

	return (
		<TableContainer>
			<TableVirtuoso
				style={{ overflowY: "scroll" }}
				components={tableComponents}
				itemContent={itemContent}
				fixedHeaderContent={renderHeaders}
				{...props}
			/>
		</TableContainer>
	);
}
