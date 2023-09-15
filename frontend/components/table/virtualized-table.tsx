import { Card } from "@mantine/core";
import { useCallback } from "react";
import { TableVirtuoso, TableVirtuosoProps } from "react-virtuoso";
import { TableHead, TableHeader } from "./table-head";
import { TableSort } from "@hooks/use-table-sort";

interface Props<TItem, TKey extends keyof TItem, ItemData = any, Context = any>
  extends TableVirtuosoProps<ItemData, Context> {
  headerItems: TableHeader<TItem, TKey>[];
  onChangeSort?: (sort: TKey) => void;
  sort?: TableSort<TItem, TKey>;
}

export function VirtualizedTable<
  TItem,
  TKey extends keyof TItem,
  ItemData = any,
  Context = any
>({
  headerItems,
  sort,
  onChangeSort,
  ...props
}: Props<TItem, TKey, ItemData, Context>) {
  const renderHeaders = useCallback(
    () => (
      <TableHead
        headers={headerItems}
        sort={sort}
        onChangeSort={onChangeSort}
      />
    ),
    [headerItems, sort, onChangeSort]
  );

  return (
    <Card padding={0} sx={{ flex: 1 }}>
      <TableVirtuoso
        {...props}
        fixedHeaderContent={renderHeaders}
        // eslint-disable-next-line react/forbid-component-props
        style={{ height: "100%" }}
      />
    </Card>
  );
}
