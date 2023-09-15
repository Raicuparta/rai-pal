import { TableHeader } from "@components/table/table-head";
import { useMemo, useState } from "react";
import { useTableSort } from "./use-table-sort";

export function useFilteredList<TItem, TKey extends keyof TItem, TFilter>(
  tableHeaders: TableHeader<TItem, TKey>[],
  data: TItem[],
  filterFunction: (item: TItem, filterValue: TFilter) => boolean,
  defaultFilterValue: TFilter
) {
  const [sort, setSort] = useTableSort<TItem, TKey>(tableHeaders[0].id);
  const [filter, setFilter] = useState<TFilter>(defaultFilterValue);

  const updateFilter = (newFilter: Partial<TFilter>) =>
    setFilter((previousFilter) => ({ ...previousFilter, ...newFilter }));

  const filteredData = useMemo(() => {
    const sortHeader = tableHeaders.find((header) => header.id === sort.id);

    return data
      .filter((item) => filterFunction(item, filter))
      .sort((gameA, gameB) => {
        const multiplier = sort.reverse ? -1 : 1;

        if (sortHeader?.customSort) {
          return multiplier * sortHeader.customSort(gameA, gameB);
        }

        const valueA = gameA[sort.id];
        const valueB = gameB[sort.id];
        if (typeof valueA === "string" && typeof valueB === "string") {
          return multiplier * valueA.localeCompare(valueB);
        } else {
          return multiplier * `${valueA}`.localeCompare(`${valueB}`);
        }
      });
  }, [data, sort, tableHeaders, filterFunction]);

  return [filteredData, sort, setSort, filter, updateFilter] as const;
}
