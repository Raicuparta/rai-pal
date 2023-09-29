import { Badge, DefaultMantineColor, Table } from "@mantine/core";
import {
	Architecture,
	Game,
	OperatingSystem,
	UnityScriptingBackend,
} from "@api/bindings";
import { GameName } from "./game-name";

type ColorRecord<T extends string = string> = Record<T, DefaultMantineColor>;

const operatingSystemColor: ColorRecord<OperatingSystem> = {
	Linux: "yellow",
	Windows: "lime",
	Unknown: "dark",
} as const;

const architectureColor: ColorRecord<Architecture> = {
	X64: "blue",
	X86: "teal",
	Unknown: "dark",
} as const;

const scriptingBackendColor: ColorRecord<UnityScriptingBackend> = {
	Il2Cpp: "red",
	Mono: "grape",
} as const;

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
				{game.engine.brand} {game.engine.version?.display ?? "Unknown"}
			</Table.Td>
		</>
	);
}
