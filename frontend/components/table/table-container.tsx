import styles from "./table.module.css";
import { Card, CardProps } from "@mantine/core";

interface Props extends CardProps {
	singleItem?: boolean;
}

export function TableContainer({ className, singleItem, ...props }: Props) {
	return (
		<Card
			className={`${className ?? ""} ${styles.table} ${singleItem ? styles.singleItem : ""}`}
			{...props}
		/>
	);
}
