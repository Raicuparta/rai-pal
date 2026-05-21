import { ComponentProps } from "react";
import styles from "./table.module.css";
import { Box } from "@mantine/core";

export function TableContainer({ className, ...props }: ComponentProps<"div">) {
	return (
		<Box
			className={`${className ?? ""} ${styles.table}`}
			{...props}
		/>
	);
}
