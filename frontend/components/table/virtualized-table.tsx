import { Card } from "@mantine/core";
import { TableVirtuoso } from "react-virtuoso";

export const VirtualizedTable: typeof TableVirtuoso = (props) => (
  <Card padding={0} sx={{ flex: 1 }}>
    <TableVirtuoso
      {...props}
      // eslint-disable-next-line react/forbid-component-props
      style={{ height: "100%" }}
    />
  </Card>
);
