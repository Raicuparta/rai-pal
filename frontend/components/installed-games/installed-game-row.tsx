import { Badge, Table } from "@mantine/core";
import { Game } from "@api/bindings";
import { GameName } from "./game-name";
import {
	architectureColor,
	engineColor,
	operatingSystemColor,
	scriptingBackendColor,
} from "../../util/color";

export function InstalledGameRow(_: number, game: Game) {
	return (
		<>
			<Table.Td>
				<GameName game={game} />
			</Table.Td>
			<Table.Td>
				<Badge color={operatingSystemColor[game.operatingSystem]}>
					{game.operatingSystem}
				</Badge>
			</Table.Td>
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
				<Badge color={engineColor[game.engine.brand]}>
					{game.engine.brand} {game.engine.version?.display ?? "Unknown"}
				</Badge>
			</Table.Td>
		</>
	);
}
