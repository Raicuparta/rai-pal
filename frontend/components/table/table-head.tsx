import { Box, Flex } from "@mantine/core";
import { MdArrowDropDown, MdArrowDropUp } from "react-icons/md";

type TableHeader<THeaderId extends string> = {
  id: THeaderId;
  label: string;
  width?: number;
};

type TableSort<THeaderId extends string> = {
  id: THeaderId;
  reverse: boolean;
};

type Props<THeaderId extends string> = {
  headers: TableHeader<THeaderId>[];
  onChangeSort: (sort: TableSort<THeaderId>) => void;
  sort: TableSort<THeaderId>;
};

export function TableHead<THeaderId extends string>(props: Props<THeaderId>) {
  const updateSort = (sortId: THeaderId) => {
    props.onChangeSort({
      id: sortId,
      reverse: props.sort.id == sortId && !props.sort.reverse,
    });
  };

  return (
    <Box component="tr" sx={(theme) => ({ background: theme.colors.dark[9] })}>
      {props.headers.map((header) => (
        <Box
          key={header.id}
          component="th"
          w={header.width}
          onClick={() => updateSort(header.id)}
          sx={(theme) => ({
            cursor: "pointer",
            ":hover": {
              background: theme.colors.dark[7],
            },
          })}
        >
          <Flex align="center">
            {header.label}
            {props.sort.id === header.id &&
              (props.sort.reverse ? <MdArrowDropDown /> : <MdArrowDropUp />)}
          </Flex>
        </Box>
      ))}
    </Box>
  );
}
