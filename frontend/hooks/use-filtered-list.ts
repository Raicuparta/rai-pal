import { TableColumn } from "@components/table/table-head";
import { useMemo, useState } from "react";
import { useTableSort } from "./use-table-sort";
import { usePersistedState } from "./use-persisted-state";

export function useFilteredList<TKey extends string, TItem, TFilter>(
	id: string,
	tableHeaders: TableColumn<TKey, TItem>[],
	data: Record<string, TItem>,
	filterFunction: (
		item: TItem,
		filterValue: TFilter,
		search: string,
	) => boolean,
	defaultFilterValue: TFilter,
) {
	const [sort, setSort] = useTableSort(
		tableHeaders.find((header) => header.sort || header.getSortValue)?.id,
	);
	const [filter, setFilter] = usePersistedState<TFilter>(
		defaultFilterValue,
		id,
	);
	const [search, setSearch] = useState("");

	const updateFilter = (newFilter: Partial<TFilter> | undefined) =>
		setFilter(
			newFilter
				? (previousFilter) => ({ ...previousFilter, ...newFilter })
				: defaultFilterValue,
		);

	const filteredData = useMemo(() => {
		const sortHeader = tableHeaders.find((header) => header.id === sort.id);

		return Object.values(data)
			.filter((item) => filterFunction(item, filter, search))
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
	}, [
		tableHeaders,
		data,
		sort.id,
		sort.reverse,
		filterFunction,
		filter,
		search,
	]);

	return [
		filteredData,
		sort,
		setSort,
		filter,
		updateFilter,
		search,
		setSearch,
	] as const;
}
