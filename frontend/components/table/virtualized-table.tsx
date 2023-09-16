import { Card } from "@mantine/core";
import { useCallback, useMemo } from "react";
import { TableVirtuoso, TableVirtuosoProps } from "react-virtuoso";
import { TableHead, TableHeader } from "./table-head";
import { TableSort } from "@hooks/use-table-sort";
import { getTableComponents } from "./table-components";

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
  Context = any
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
        sort={sort}
        onChangeSort={onChangeSort}
      />
    ),
    [headerItems, sort, onChangeSort]
  );

  const tableComponents = useMemo(
    () => getTableComponents(onClickItem),
    [onClickItem]
  );

  return (
    <Card padding={0} sx={{ flex: 1 }}>
      <TableVirtuoso
        fixedHeaderContent={renderHeaders}
        components={tableComponents}
        // eslint-disable-next-line react/forbid-component-props
        style={{ height: "100%" }}
        {...props}
      />
    </Card>
  );
}
