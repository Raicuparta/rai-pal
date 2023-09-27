import classes from "./table.module.css";
import { Card } from "@mantine/core";

type Props = {
  children: React.ReactNode;
};

export const TableContainer = (props: Props) => (
  <Card className={classes.table}>{props.children}</Card>
);
