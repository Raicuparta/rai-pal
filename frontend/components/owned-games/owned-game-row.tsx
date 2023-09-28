import { OwnedGame } from "@api/bindings";
import { Badge, Table } from "@mantine/core";
import { engineColor } from "../../util/color";
import { IconCheck } from "@tabler/icons-react";

export function OwnedGameRow(_: number, ownedUnityGame: OwnedGame) {
	return (
		<>
			<Table.Td>{ownedUnityGame.name}</Table.Td>
			<Table.Td align="center">
				<Badge color={engineColor[ownedUnityGame.engine]}>
					{ownedUnityGame.engine}
				</Badge>
			</Table.Td>
			<Table.Td align="center">
				{ownedUnityGame.osList.includes("Linux") ? <IconCheck size={15} /> : ""}
			</Table.Td>
			<Table.Td align="center">
				{ownedUnityGame.installed ? <IconCheck size={15} /> : ""}
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
