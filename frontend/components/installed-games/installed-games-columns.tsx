import { Table, Tooltip } from "@mantine/core";
import {
	AppType,
	Architecture,
	EngineBrand,
	GameMode,
	ProviderId,
	UnityScriptingBackend,
} from "@api/bindings";
import { TableColumnBase, columnMapToList } from "@components/table/table-head";
import { ItemName } from "../item-name";
import {
	AppTypeBadge,
	ArchitectureBadge,
	EngineBadge,
	GameModeBadge,
	ProviderBadge,
	UnityBackendBadge,
} from "@components/badges/color-coded-badge";
import { ThumbnailCell } from "@components/table/thumbnail-cell";
import { OutdatedMarker } from "@components/outdated-marker";
import {
	appTypeFilterOptions,
	engineFilterOptions,
	providerFilterOptions,
} from "../../util/common-filter-options";
import styles from "../table/table.module.css";
import { ProcessedInstalledGame } from "@hooks/use-processed-installed-games";
import { getThumbnailWithFallback } from "../../util/fallback-thumbnail";
import { sortGamesByEngine } from "../../util/game-engines";

const thumbnail: TableColumnBase<ProcessedInstalledGame> = {
	hideInDetails: true,
	label: "Thumbnail",
	hideLabel: true,
	hidable: true,
	width: 100,
	renderCell: (game) => (
		<ThumbnailCell
			src={getThumbnailWithFallback(
				game.thumbnailUrl || game.ownedGame?.thumbnailUrl,
				game.provider,
			)}
		/>
	),
};

const name: TableColumnBase<ProcessedInstalledGame> = {
	hideInDetails: true,
	label: "Game",
	getSortValue: (game) => game.title.display,
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
						{game.title.display}
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
	getSortValue: (game) => game.provider,
	getFilterValue: (game) => game.provider,
	filterOptions: providerFilterOptions,
	renderCell: (game) => (
		<Table.Td>
			<ProviderBadge value={game.provider} />
		</Table.Td>
	),
};

const architecture: TableColumnBase<ProcessedInstalledGame, Architecture> = {
	label: "Arch",
	width: 70,
	center: true,
	hidable: true,
	getSortValue: (game) => game.executable.architecture,
	getFilterValue: (game) => game.executable.architecture,
	filterOptions: [
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
	getFilterValue: (game) => game.executable.scriptingBackend,
	filterOptions: [
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
	getSortValue: (game) => game.ownedGame?.gameMode,
	getFilterValue: (game) => game.ownedGame?.gameMode ?? null,
	filterOptions: [
		{ label: "Flat", value: "Flat" },
		{ label: "VR", value: "VR" },
	],
	renderCell: (game) => (
		<Table.Td>
			<GameModeBadge value={game.ownedGame?.gameMode} />
		</Table.Td>
	),
};

const getAppType = (game: ProcessedInstalledGame) =>
	game.ownedGame?.appType ?? null;

const appType: TableColumnBase<ProcessedInstalledGame, AppType> = {
	label: "App Type",
	width: 110,
	center: true,
	hidable: true,
	getSortValue: (game) => getAppType(game),
	getFilterValue: (game) => getAppType(game),
	filterOptions: appTypeFilterOptions,
	renderCell: (game) => (
		<Table.Td>
			<AppTypeBadge value={getAppType(game)}>{getAppType(game)}</AppTypeBadge>
		</Table.Td>
	),
};

const engine: TableColumnBase<ProcessedInstalledGame, EngineBrand> = {
	label: "Engine",
	width: 180,
	center: true,
	hidable: true,
	sort: (dataA, dataB) =>
		sortGamesByEngine(dataA.executable.engine, dataB.executable.engine),
	getFilterValue: (game) => game.executable.engine?.brand ?? null,
	unavailableValues: ["Godot", "GameMaker"],
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
				label={engine ? (engine.version?.display ?? "-") : undefined}
			/>
		</Table.Td>
	),
};

const installedGamesColumnsMap = {
	thumbnail,
	name,
	provider,
	gameMode,
	architecture,
	scriptingBackend,
	engine,
	appType,
};

export type InstalledGameColumnsId = keyof typeof installedGamesColumnsMap;

export const installedGamesColumns = columnMapToList(installedGamesColumnsMap);
