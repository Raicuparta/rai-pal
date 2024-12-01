import { useCallback, useMemo } from "react";
import { TableVirtuoso, TableVirtuosoProps } from "react-virtuoso";
import { TableHead, TableColumn } from "./table-head";
import { TableSort } from "@hooks/use-table-sort";
import { getTableComponents } from "./table-components";
import { TableContainer } from "./table-container";
import { useVirtuosoHeaderContent } from "@hooks/use-virtuoso-header-content";
import { useVirtuosoTableComponents } from "@hooks/use-virtuoso-table-components";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
interface Props<TKey extends string, TItem, Context = any>
	extends TableVirtuosoProps<TItem, Context> {
	readonly columns: TableColumn<TKey, unknown>[];
	readonly onChangeSort?: (sort: string) => void;
	readonly sort?: TableSort;
	readonly onClickItem: (item: TItem) => void;
}

export function VirtualizedTable<
	TKey extends string,
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	Context = any,
>({
	columns,
	sort,
	onChangeSort,
	onClickItem,
	...props
}: Props<TKey, Context>) {
	const renderHeaders = useVirtuosoHeaderContent(columns, onChangeSort, sort);

	const tableComponents = useVirtuosoTableComponents();

	return (
		<TableContainer>
			<TableVirtuoso
				style={{ overflowY: "scroll" }}
				components={tableComponents}
				fixedHeaderContent={renderHeaders}
				defaultItemHeight={33}
				{...props}
			/>
		</TableContainer>
	);
}
