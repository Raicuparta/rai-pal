import { Badge, Table, Box } from "@mantine/core";
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
				<Badge color={architectureColor[game.architecture]}>
					{game.architecture}
				</Badge>
			</Table.Td>
			<Table.Td>
				<Badge color={scriptingBackendColor[game.scriptingBackend]}>
					{game.scriptingBackend}
				</Badge>
			</Table.Td>
			<Table.Td>
				<Badge color={game.engine ? engineColor[game.engine.brand] : "dark"}>
					{game.engine?.brand ?? "Unknown"}{" "}
					<Box
						component="small"
						opacity={0.5}
					>
						{game.engine?.version?.display ?? ""}
					</Box>
				</Badge>
			</Table.Td>
		</>
	);
}
