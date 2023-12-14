import { Table, Tooltip } from "@mantine/core";
import {
	Architecture,
	GameEngine,
	GameEngineBrand,
	GameEngineVersion,
	GameMode,
	OperatingSystem,
	ProviderId,
	UnityScriptingBackend,
} from "@api/bindings";
import { TableColumnBase, columnMapToList } from "@components/table/table-head";
import { ItemName } from "../item-name";
import {
	ArchitectureBadge,
	EngineBadge,
	GameModeBadge,
	OperatingSystemBadge,
	ProviderBadge,
	UnityBackendBadge,
} from "@components/badges/color-coded-badge";
import { ThumbnailCell } from "@components/table/thumbnail-cell";
import { OutdatedMarker } from "@components/OutdatedMarker";
import {
	engineFilterOptions,
	providerFilterOptions,
} from "../../util/common-filter-options";
import styles from "../table/table.module.css";
import { ProcessedInstalledGame } from "@hooks/use-processed-installed-games";

const thumbnail: TableColumnBase<ProcessedInstalledGame> = {
	hideInDetails: true,
	label: "Thumbnail",
	hideLabel: true,
	hidable: true,
	width: 100,
	renderCell: (game) => <ThumbnailCell url={game.thumbnailUrl} />,
};

const name: TableColumnBase<ProcessedInstalledGame> = {
	hideInDetails: true,
	label: "Game",
	getSortValue: (game) => game.name,
	renderCell: (game) => (
		<Table.Td className={styles.nameCell}>
			<Tooltip
				disabled={!game.hasOutdatedMod}
				label="One of the mods installed in this game is outdated."
				position="bottom"
			>
				<span>
					<ItemName label={game.discriminator}>
						{game.hasOutdatedMod && <OutdatedMarker />}
						{game.name}
					</ItemName>
				</span>
			</Tooltip>
		</Table.Td>
	),
};

const provider: TableColumnBase<ProcessedInstalledGame, ProviderId> = {
	label: "Provider",
	width: 110,
	center: true,
	hidable: true,
	getSortValue: (game) => game.providerId,
	filterOptions: providerFilterOptions,
	renderCell: (game) => (
		<Table.Td>
			<ProviderBadge value={game.providerId} />
		</Table.Td>
	),
};

const operatingSystem: TableColumnBase<
	ProcessedInstalledGame,
	OperatingSystem
> = {
	label: "OS",
	width: 110,
	center: true,
	hidable: true,
	getSortValue: (game) => game.executable.operatingSystem,
	filterOptions: [
		{ label: "Any OS", value: "" },
		{ label: "Windows", value: "Windows" },
		{ label: "Linux", value: "Linux" },
	],
	renderCell: (game) => (
		<Table.Td>
			<OperatingSystemBadge value={game.executable.operatingSystem} />
		</Table.Td>
	),
};

const architecture: TableColumnBase<ProcessedInstalledGame, Architecture> = {
	label: "Arch",
	width: 70,
	center: true,
	hidable: true,
	getSortValue: (game) => game.executable.architecture,
	filterOptions: [
		{ label: "Any architecture", value: "" },
		{ label: "x64", value: "X64" },
		{ label: "x86", value: "X86" },
	],
	renderCell: (game) => (
		<Table.Td>
			<ArchitectureBadge value={game.executable.architecture} />
		</Table.Td>
	),
};

const scriptingBackend: TableColumnBase<
	ProcessedInstalledGame,
	UnityScriptingBackend
> = {
	label: "Backend",
	width: 90,
	center: true,
	hidable: true,
	getSortValue: (game) => game.executable.scriptingBackend,
	filterOptions: [
		{ label: "Any backend", value: "" },
		{ label: "IL2CPP", value: "Il2Cpp" },
		{ label: "Mono", value: "Mono" },
	],
	renderCell: (game) => (
		<Table.Td>
			<UnityBackendBadge value={game.executable.scriptingBackend} />
		</Table.Td>
	),
};

const gameMode: TableColumnBase<ProcessedInstalledGame, GameMode> = {
	label: "Mode",
	width: 90,
	center: true,
	hidable: true,
	getSortValue: (game) => game.gameMode,
	filterOptions: [
		{ label: "Any mode", value: "" },
		{ label: "Flat", value: "Flat" },
		{ label: "VR", value: "VR" },
	],
	renderCell: (game) => (
		<Table.Td>
			<GameModeBadge value={game.gameMode} />
		</Table.Td>
	),
};

const defaultVersion: GameEngineVersion = {
	major: 0,
	minor: 0,
	patch: 0,
	suffix: "",
	display: "",
};

function getAdjustedMajor(engine: GameEngine | null) {
	const major = engine?.version?.major;
	if (!major) return 0;

	if (engine?.brand === "Unity" && major > 5 && major < 2000) {
		// Unity did this silly thing.
		// It went from Unity 5 to Unity 2017-2023, then back to Unity 6.
		// So for sorting purposes, we consider Unity 6, 7, 8, etc to be Unity 2106, 2107, 2108, etc.
		// This will of course break if they go back to year-based versions,
		// or if they release a LOT of major versions.
		return major + 2100;
	}

	return major;
}

const engine: TableColumnBase<ProcessedInstalledGame, GameEngineBrand> = {
	label: "Engine",
	width: 180,
	center: true,
	hidable: true,
	sort: (dataA, dataB) => {
		const engineA = dataA.executable.engine;
		const engineB = dataB.executable.engine;
		const versionA = engineA?.version ?? defaultVersion;
		const versionB = engineB?.version ?? defaultVersion;
		const brandA = engineA?.brand ?? "";
		const brandB = engineB?.brand ?? "";

		return (
			brandA.localeCompare(brandB) ||
			getAdjustedMajor(engineA) - getAdjustedMajor(engineB) ||
			versionA.minor - versionB.minor ||
			versionA.patch - versionB.patch ||
			0
		);
	},
	getFilterValue: (game) => game.executable.engine?.brand ?? "",
	filterOptions: engineFilterOptions,
	renderCell: ({ executable: { engine } }) => (
		<Table.Td
			// A bit annoying that I'm defining the column width in two places (see engineColumn.width),
			// but it's to prevent this one from being squished and hiding the version number.
			// Maybe I shouldn't be using a regular table component at all for this...
			miw={170}
		>
			<EngineBadge
				maw={70}
				value={engine?.brand}
				label={engine ? engine.version?.display ?? "-" : undefined}
			/>
		</Table.Td>
	),
};

const installedGamesColumnsMap = {
	thumbnail,
	name,
	provider,
	gameMode,
	operatingSystem,
	architecture,
	scriptingBackend,
	engine,
};

export type InstalledGameColumnsId = keyof typeof installedGamesColumnsMap;

export const installedGamesColumns = columnMapToList(installedGamesColumnsMap);
