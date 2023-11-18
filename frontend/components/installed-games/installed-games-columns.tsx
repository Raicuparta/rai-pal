import { Flex, Table } from "@mantine/core";
import { InstalledGame } from "@api/bindings";
import { TableColumn } from "@components/table/table-head";
import { GameName } from "./game-name";
import {
	ArchitectureBadge,
	EngineBadge,
	OperatingSystemBadge,
	UnityBackendBadge,
} from "@components/badges/color-coded-badge";
import { ThumbnailCell } from "@components/table/thumbnail-cell";

export const installedGamesColumns: TableColumn<InstalledGame>[] = [
	{
		id: "thumbnailUrl",
		label: "",
		width: 100,
		renderCell: (game) => <ThumbnailCell url={game.thumbnailUrl} />,
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
		getSortValue: (game) => {
			const engine = game.executable.engine;
			if (!engine?.version) return 0;

			let major = engine.version.major;

			// Unity did this silly thing.
			// It went from Unity 5 to Unity 2017-2023, then back to Unity 6.
			// So for sorting purposes, we consider Unity 6, 7, 8, etc to be Unity 2106, 2107, 2108, etc.
			// This will of course break if they go back to year-based versions.
			if (engine.brand == "Unity" && major > 5 && major < 2000) {
				major += 2100;
			}

			return (
				major * 100000000 + engine.version.minor * 100000 + engine.version.patch
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
