import { Badge, Table, Box, Flex } from "@mantine/core";
import { Game } from "@api/bindings";
import { GameName } from "./game-name";
import {
	architectureColor,
	engineColor,
	// operatingSystemColor,
	scriptingBackendColor,
} from "../../util/color";
import { GameThumbnail } from "@components/game-thumbnail";

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
				<Badge
					color={
						game.scriptingBackend
							? scriptingBackendColor[game.scriptingBackend]
							: "dark"
					}
				>
					{game.scriptingBackend ?? "-"}
				</Badge>
			</Table.Td>
			<Table.Td>
				<Flex
					align="center"
					gap="xs"
				>
					<Badge
						fullWidth={false}
						color={game.engine ? engineColor[game.engine.brand] : "dark"}
					>
						{game.engine?.brand ?? "Unknown"}{" "}
					</Badge>
					{game.engine?.version && (
						<Box
							component="small"
							opacity={0.5}
						>
							{game.engine.version.display}
						</Box>
					)}
				</Flex>
			</Table.Td>
		</>
	);
}
