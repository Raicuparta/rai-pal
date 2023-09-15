import { TableHeader } from "@components/table/table-head";
import { useMemo } from "react";
import { TableSort } from "./use-table-sort";

export function useFilteredList<TItem, TKey extends keyof TItem>(
  tableHeaders: TableHeader<TItem, TKey>[],
  data: TItem[],
  filter: (item: TItem) => boolean,
  sort: TableSort<TItem, TKey>
) {
  return useMemo(() => {
    const sortHeader = tableHeaders.find((header) => header.id === sort.id);

    return data.filter(filter).sort((gameA, gameB) => {
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
  }, [filter, data, sort, tableHeaders]);
}
