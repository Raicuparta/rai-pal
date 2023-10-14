import { TableColumn } from "@components/table/table-head";
import { useMemo, useState } from "react";
import { useTableSort } from "./use-table-sort";

export type Filter = {
	search: string;
};

export function useFilteredList<TItem, TFilter extends Filter>(
	tableHeaders: TableColumn<TItem>[],
	data: TItem[],
	filterFunction: (item: TItem, filterValue: TFilter) => boolean,
	defaultFilterValue: TFilter,
) {
	const [sort, setSort] = useTableSort(
		tableHeaders.find((header) => header.sort || header.getSortValue)?.id,
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

				if (sortHeader?.sort) {
					return multiplier * sortHeader.sort(gameA, gameB);
				} else if (sortHeader?.getSortValue) {
					const valueA = sortHeader.getSortValue(gameA);
					const valueB = sortHeader.getSortValue(gameB);

					if (typeof valueA === "string" && typeof valueB === "string") {
						return multiplier * valueA.localeCompare(valueB);
					} else if (typeof valueA === "number" && typeof valueB === "number") {
						return multiplier * (valueA - valueB);
					} else if (
						typeof valueA === "boolean" &&
						typeof valueB === "boolean"
					) {
						return multiplier * ((valueA ? 0 : 1) - (valueB ? 0 : 1));
					} else {
						return multiplier * `${valueA}`.localeCompare(`${valueB}`);
					}
				}

				return 0;
			});
	}, [tableHeaders, data, sort.id, sort.reverse, filterFunction, filter]);

	return [filteredData, sort, setSort, filter, updateFilter] as const;
}
