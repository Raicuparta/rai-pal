import styles from "./table.module.css";
import { Card, CardProps } from "@mantine/core";

export function TableContainer({ className, ...props }: CardProps) {
	return (
		<Card
			className={`${className ?? ""} ${styles.table}`}
			{...props}
		/>
	);
}
