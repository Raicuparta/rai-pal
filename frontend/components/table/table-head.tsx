import { Box, Flex } from "@mantine/core";
import { MdArrowDropDown, MdArrowDropUp } from "react-icons/md";

export type TableHeader<THeaderId extends string> = {
  id: THeaderId;
  label: string;
  width?: number;
  customSort?: (a: Record<THeaderId, any>, b: Record<THeaderId, any>) => number;
};

type TableSort<THeaderId extends string> = {
  id: THeaderId;
  reverse: boolean;
};

type Props<THeaderId extends string> = {
  headers: TableHeader<THeaderId>[];
  onChangeSort?: (sort: THeaderId) => void;
  sort?: TableSort<THeaderId>;
};

export function TableHead<THeaderId extends string>(props: Props<THeaderId>) {
  return (
    <Box component="tr" sx={(theme) => ({ background: theme.colors.dark[9] })}>
      {props.headers.map((header) => (
        <Box
          key={header.id}
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
