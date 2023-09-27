import { Box, Flex, Table } from "@mantine/core";
import { MdArrowDropDown, MdArrowDropUp } from "react-icons/md";
import classes from "./table.module.css";

export type TableHeader<TItem, TKey extends keyof TItem> = {
  id: TKey;
  label: string;
  width?: number;
  center?: boolean;
  customSort?: (itemA: TItem, itemB: TItem) => number;
};

type TableSort<TItem, TKey extends keyof TItem> = {
  id: TKey;
  reverse: boolean;
};

type Props<TItem, TKey extends keyof TItem> = {
  readonly headers: TableHeader<TItem, TKey>[];
  readonly onChangeSort?: (sort: TKey) => void;
  readonly sort?: TableSort<TItem, TKey>;
};

export function TableHead<TItem, TKey extends keyof TItem>(
  props: Props<TItem, TKey>
) {
  return (
    <Table.Tr>
      {props.headers.map((header) => (
        <Table.Th
          key={String(header.id)}
          w={header.width}
          className={props.onChangeSort ? classes.sortable : undefined}
          onClick={() =>
            props.onChangeSort ? props.onChangeSort(header.id) : undefined
          }
        >
          <Flex justify={header.center ? "center" : undefined}>
            {header.label}
            <Box w={0} h={0}>
              {props.sort?.id === header.id &&
                (props.sort.reverse ? <MdArrowDropDown /> : <MdArrowDropUp />)}
            </Box>
          </Flex>
        </Table.Th>
      ))}
    </Table.Tr>
  );
}
