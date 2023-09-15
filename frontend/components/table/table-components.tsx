import { Box, Table as MantineTable } from "@mantine/core";
import { TableComponents } from "react-virtuoso";

export function getTableComponents<TItem>(
  onClickItem: (item: TItem) => void
): TableComponents<TItem, any> {
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
