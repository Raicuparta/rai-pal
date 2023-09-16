import { Box, Table as MantineTable } from "@mantine/core";
import { TableComponents } from "react-virtuoso";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function getTableComponents<TItem, TContext = any>(
  onClickItem: (item: TItem) => void
): TableComponents<TItem, TContext> {
  return {
    Table: (props) => (
      <MantineTable {...props} highlightOnHover sx={{ tableLayout: "fixed" }} />
    ),
    TableRow: (props) => (
      <Box
        component="tr"
        sx={{ cursor: "pointer" }}
        onClick={() => onClickItem(props.item)}
        {...props}
      />
    ),
  };
}
