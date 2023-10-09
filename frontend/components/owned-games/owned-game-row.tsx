import { OwnedGame } from "@api/bindings";
import { Flex, Table } from "@mantine/core";
import { IconCheck } from "@tabler/icons-react";
import { GameThumbnail } from "@components/game-thumbnail";
import { EngineBadge } from "@components/color-coded-badge";
import styles from "../table/table.module.css";

export function OwnedGameRow(_: number, ownedUnityGame: OwnedGame) {
	return (
		<>
			<GameThumbnail url={ownedUnityGame.thumbnailUrl} />
			<Table.Td className={styles.leftAligned}>
				<Flex>{ownedUnityGame.name}</Flex>
			</Table.Td>
			<Table.Td align="center">
				<EngineBadge value={ownedUnityGame.engine} />
			</Table.Td>
			{/* <Table.Td align="center">
				{ownedUnityGame.osList.includes("Linux") ? <IconCheck /> : ""}
			</Table.Td> */}
			<Table.Td align="center">
				{ownedUnityGame.installed ? <IconCheck /> : ""}
			</Table.Td>
			<Table.Td align="center">
				{ownedUnityGame.releaseDate
					? new Date(ownedUnityGame.releaseDate * 1000)
							.toISOString()
							.split("T")[0]
					: "Unknown"}
			</Table.Td>
		</>
	);
}
