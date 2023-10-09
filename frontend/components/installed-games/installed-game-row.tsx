import { Badge, Table, Box, Flex } from "@mantine/core";
import { Game } from "@api/bindings";
import { GameName } from "./game-name";
import {
	architectureColor,
	// operatingSystemColor,
} from "../../util/color";
import { GameThumbnail } from "@components/game-thumbnail";
import { EngineBadge } from "@components/badges/engine-badge";
import { UnityBackendBadge } from "@components/badges/unity-backend.badge";

export function InstalledGameRow(_: number, game: Game) {
	return (
		<>
			<GameThumbnail url={game.thumbnailUrl} />
			<Table.Td>
				<GameName game={game} />
			</Table.Td>
			{/* <Table.Td>
				<Badge color={operatingSystemColor[game.operatingSystem]}>
					{game.operatingSystem}
				</Badge>
			</Table.Td> */}
			<Table.Td>
				<Badge
					color={
						game.architecture ? architectureColor[game.architecture] : "dark"
					}
				>
					{game.architecture ?? "X??"}
				</Badge>
			</Table.Td>
			<Table.Td>
				<UnityBackendBadge backend={game.scriptingBackend} />
			</Table.Td>
			<Table.Td>
				<Flex
					align="center"
					gap="xs"
				>
					<EngineBadge engine={game.engine?.brand} />
					{game.engine?.version && (
						<Box
							component="small"
							opacity={0.75}
						>
							{game.engine.version.display}
						</Box>
					)}
				</Flex>
			</Table.Td>
		</>
	);
}
