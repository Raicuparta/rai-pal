import { Table } from "@mantine/core";
import styles from "./table.module.css";

type Props = {
	readonly src: string;
};

export function ThumbnailCell(props: Props) {
	return (
		<Table.Td
			className={styles.thumbnail}
			style={{
				backgroundImage: `url(${props.src})`,
			}}
		/>
	);
}
