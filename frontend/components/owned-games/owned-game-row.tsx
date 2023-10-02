import { OwnedGame } from "@api/bindings";
import { Badge, Flex, Table } from "@mantine/core";
import { engineColor } from "../../util/color";
import { IconCheck } from "@tabler/icons-react";
import { GameThumbnail } from "@components/game-thumbnail";

export function OwnedGameRow(_: number, ownedUnityGame: OwnedGame) {
	return (
		<>
			<Table.Td p={0}>
				<Flex
					gap="sm"
					align="center"
				>
					<GameThumbnail
						url={`https://cdn.cloudflare.steamstatic.com/steam/apps/${ownedUnityGame.id}/capsule_231x87.jpg`}
					/>
					{ownedUnityGame.name}
				</Flex>
			</Table.Td>
			<Table.Td align="center">
				<Badge color={engineColor[ownedUnityGame.engine]}>
					{ownedUnityGame.engine}
				</Badge>
			</Table.Td>
			<Table.Td align="center">
				{ownedUnityGame.osList.includes("Linux") ? <IconCheck /> : ""}
			</Table.Td>
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
