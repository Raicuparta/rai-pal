import type { ComponentType } from "react";

export type TableColumnBase<TItem> = {
	component: ComponentType<{ item: TItem }>;
	width?: number;
};

export interface TableColumn<
	TKey extends string,
	TItem,
> extends TableColumnBase<TItem> {
	id: TKey;
}

export function columnMapToList<TItem, TKey extends string>(
	columnMap: Record<TKey, TableColumnBase<TItem>>,
): TableColumn<TKey, TItem>[] {
	return Object.entries<TableColumnBase<TItem>>(columnMap).map(
		([id, column]) => ({
			...column,
			id: id as TKey,
		}),
	);
}
