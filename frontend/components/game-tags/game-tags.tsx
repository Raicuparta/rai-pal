import { Game } from "@api/bindings";
import { Table } from "@mantine/core";
import styles from "./game-tags.module.css";

export function GameTagsCell({ item }: { readonly item: Game }) {
	return (
		<Table.Td p={0}>
			<div className={styles.wrapper}>
				{item?.tags.sort().map((tag) => (
					<span
						className={styles.tag}
						key={tag}
					>
						{tag}
					</span>
				))}
			</div>
		</Table.Td>
	);
}
