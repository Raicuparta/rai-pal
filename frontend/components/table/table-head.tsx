import { Box, Flex } from "@mantine/core";
import { MdArrowDropDown, MdArrowDropUp } from "react-icons/md";

export type TableHeader<TItem, TKey extends keyof TItem> = {
  id: TKey;
  label: string;
  width?: number;
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
    <Box component="tr" sx={(theme) => ({ background: theme.colors.dark[9] })}>
      {props.headers.map((header) => (
        <Box
          key={String(header.id)}
          component="th"
          w={header.width}
          onClick={() =>
            props.onChangeSort ? props.onChangeSort(header.id) : undefined
          }
          sx={
            props.onChangeSort
              ? (theme) => ({
                  cursor: "pointer",
                  ":hover": {
                    background: theme.colors.dark[7],
                  },
                })
              : undefined
          }
        >
          <Flex align="center">
            {header.label}
            {props.sort?.id === header.id &&
              (props.sort.reverse ? <MdArrowDropDown /> : <MdArrowDropUp />)}
          </Flex>
        </Box>
      ))}
    </Box>
  );
}
