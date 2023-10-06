import { TableHeader } from "@components/table/table-head";
import { useMemo, useState } from "react";
import { useTableSort } from "./use-table-sort";

export type Filter = {
	search: string;
};

export function useFilteredList<
	TItem,
	TKey extends keyof TItem,
	TFilter extends Filter,
>(
	tableHeaders: TableHeader<TItem, TKey>[],
	data: TItem[],
	filterFunction: (item: TItem, filterValue: TFilter) => boolean,
	defaultFilterValue: TFilter,
) {
	const [sort, setSort] = useTableSort<TItem, TKey>(
		tableHeaders.find((header) => header.sortable)?.id,
	);
	const [filter, setFilter] = useState<TFilter>(defaultFilterValue);

	const updateFilter = (newFilter: Partial<TFilter> | undefined) =>
		setFilter(
			newFilter
				? (previousFilter) => ({ ...previousFilter, ...newFilter })
				: defaultFilterValue,
		);

	const filteredData = useMemo(() => {
		const sortHeader = tableHeaders.find((header) => header.id === sort.id);

		return data
			.filter((item) => filterFunction(item, filter))
			.sort((gameA, gameB) => {
				if (sort.id == undefined) return 0;

				const multiplier = sort.reverse ? -1 : 1;

				if (sortHeader?.customSort) {
					return multiplier * sortHeader.customSort(gameA, gameB);
				}

				const valueA = gameA[sort.id];
				const valueB = gameB[sort.id];
				if (typeof valueA === "string" && typeof valueB === "string") {
					return multiplier * valueA.localeCompare(valueB);
				} else if (typeof valueA === "number" && typeof valueB === "number") {
					return multiplier * (valueA - valueB);
				} else {
					return multiplier * `${valueA}`.localeCompare(`${valueB}`);
				}
			});
	}, [tableHeaders, data, sort.id, sort.reverse, filterFunction, filter]);

	return [filteredData, sort, setSort, filter, updateFilter] as const;
}
