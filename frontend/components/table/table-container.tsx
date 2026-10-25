import { ComponentProps } from "react";
import styles from "./table.module.css";
import { Box } from "@mantine/core";
import { ScrollAreaFill } from "@components/scroll-area-fill";

type Props = ComponentProps<"div"> & {
	// Virtualized tables render their own scroller.
	readonly scroll?: boolean;
};

export function TableContainer({ className, scroll = true, ...props }: Props) {
	const classes = `${className ?? ""} ${styles.table}`;

	if (!scroll) {
		return (
			<Box
				className={classes}
				{...props}
			/>
		);
	}

	return (
		<ScrollAreaFill
			className={classes}
			{...props}
		/>
	);
}
