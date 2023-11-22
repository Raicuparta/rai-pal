import { Table } from "@mantine/core";
import styles from "./table.module.css";

type Props = {
	readonly url: string | null;
};

export function ThumbnailCell(props: Props) {
	return (
		<Table.Td
			className={styles.thumbnail}
			style={{
				backgroundImage: `url(${props.url ?? "images/fallback-thumbnail.png"})`,
			}}
		/>
	);
}
