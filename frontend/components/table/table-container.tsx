import classes from "./table.module.css";
import { Card, CardProps } from "@mantine/core";

export function TableContainer(props: CardProps) {
	return (
		<Card
			className={classes.table}
			{...props}
		/>
	);
}
