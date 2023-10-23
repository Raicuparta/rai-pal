import { Flex, Table } from "@mantine/core";
import { InstalledGame, GameEngineVersion } from "@api/bindings";
import { TableColumn } from "@components/table/table-head";
import { GameName } from "./game-name";
import {
	ArchitectureBadge,
	EngineBadge,
	OperatingSystemBadge,
	UnityBackendBadge,
} from "@components/badges/color-coded-badge";
import { GameThumbnail } from "@components/game-thumbnail";

const defaultVersion: GameEngineVersion = {
	major: 0,
	minor: 0,
	patch: 0,
	suffix: "",
	display: "",
};

export const installedGamesColumns: TableColumn<InstalledGame>[] = [
	{
		id: "thumbnailUrl",
		label: "",
		width: 100,
		renderCell: (game) => <GameThumbnail url={game.thumbnailUrl} />,
	},
	{
		id: "name",
		label: "Game",
		width: undefined,
		getSortValue: (game) => game.name,
		renderCell: (game) => (
			<Table.Td>
				<GameName game={game} />
			</Table.Td>
		),
	},
	{
		id: "operatingSystem",
		label: "OS",
		width: 110,
		center: true,
		hidable: true,
		getSortValue: (game) => game.executable.operatingSystem,
		renderCell: (game) => (
			<Table.Td>
				<OperatingSystemBadge value={game.executable.operatingSystem} />
			</Table.Td>
		),
	},
	{
		id: "architecture",
		label: "Arch",
		width: 70,
		center: true,
		hidable: true,
		getSortValue: (game) => game.executable.architecture,
		renderCell: (game) => (
			<Table.Td>
				<ArchitectureBadge value={game.executable.architecture} />
			</Table.Td>
		),
	},
	{
		id: "scriptingBackend",
		label: "Backend",
		width: 90,
		center: true,
		hidable: true,
		getSortValue: (game) => game.executable.scriptingBackend,
		renderCell: (game) => (
			<Table.Td>
				<UnityBackendBadge value={game.executable.scriptingBackend} />
			</Table.Td>
		),
	},
	{
		id: "engine",
		label: "Engine",
		width: 170,
		center: true,
		hidable: true,
		sort: (dataA, dataB) => {
			const versionA = dataA.executable.engine?.version ?? defaultVersion;
			const versionB = dataB.executable.engine?.version ?? defaultVersion;
			const brandA = dataA.executable.engine?.brand ?? "";
			const brandB = dataB.executable.engine?.brand ?? "";

			return (
				brandA.localeCompare(brandB) ||
				versionA.major - versionB.major ||
				versionA.minor - versionB.minor ||
				versionA.patch - versionB.patch ||
				0
			);
		},
		renderCell: ({ executable: { engine } }) => (
			<Table.Td>
				<Flex
					align="center"
					gap="xs"
				>
					<EngineBadge
						maw={70}
						value={engine?.brand}
						label={engine ? engine.version?.display ?? "-" : undefined}
					/>
				</Flex>
			</Table.Td>
		),
	},
];
