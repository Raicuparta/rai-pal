import classes from "./table.module.css";
import { Card } from "@mantine/core";

type Props = {
	readonly children: React.ReactNode;
};

export function TableContainer(props: Props) {
	return <Card className={classes.table}>{props.children}</Card>;
}
