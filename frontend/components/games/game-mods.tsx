import { Alert, Divider, Table } from "@mantine/core";
import { EngineVersionRange, DbGame, commands } from "@api/bindings";
import { useCallback, useMemo } from "react";
import { CommandButton } from "@components/command-button";
import { IconTrash } from "@tabler/icons-react";
import { useAtomValue } from "jotai";
import { modLoadersAtom } from "@hooks/use-data";
import { useUnifiedMods } from "@hooks/use-unified-mods";
import { GameModRow } from "./game-mod-row";
import { TableContainer } from "@components/table/table-container";
import { useLocalization } from "@hooks/use-localization";
import { useCommandData } from "@hooks/use-command-data";
import { useAppEvent } from "@hooks/use-app-event";

type Props = {
	readonly game: DbGame;
};

function isVersionWithinRange(
	{
		engineVersionMajor: major,
		engineVersionMinor: minor,
		engineVersionPatch: patch,
	}: DbGame,
	range: EngineVersionRange | null,
) {
	if (!major || !range) return true;

	if (!major) return false;

	const { minimum, maximum } = range;

	if (minimum && minimum.major > major) return false;
	if (maximum && maximum.major < major) return false;
	if (
		minimum &&
		minimum.major === major &&
		minimum.minor != null &&
		minor != null &&
		minimum.minor > minor
	)
		return false;
	if (
		maximum &&
		maximum.major === major &&
		maximum.minor != null &&
		minor != null &&
		maximum.minor < minor
	)
		return false;
	if (
		minimum &&
		minimum.major === major &&
		minimum.minor === minor &&
		minimum.patch != null &&
		patch != null &&
		minimum.patch > patch
	)
		return false;
	if (
		maximum &&
		maximum.major === major &&
		maximum.minor === minor &&
		maximum.patch != null &&
		patch != null &&
		maximum.patch < patch
	)
		return false;

	return true;
}

const defaultInstalledModVersions: Record<string, string> = {};

export function GameMods({ game }: Props) {
	const t = useLocalization("gameModal");
	const modLoaderMap = useAtomValue(modLoadersAtom);
	const mods = useUnifiedMods();
	const getInstalledModVersions = useCallback(
		() => commands.getInstalledModVersions(game.providerId, game.gameId),
		[game],
	);
	const [installedModVersions, updateInstalledModVersions] = useCommandData(
		getInstalledModVersions,
		defaultInstalledModVersions,
		!game?.exePath,
	);
	const getRemoteConfigs = useCallback(
		() => commands.getRemoteConfigs(game.providerId, game.gameId),
		[game],
	);
	const [remoteConfigs, updateRemoteConfigs] = useCommandData(
		getRemoteConfigs,
		null,
		!game?.exePath,
	);

	useAppEvent(
		"refreshGame",
		`installed-mods-${game.providerId}:${game.gameId}`,
		([refreshedProviderId, refreshedGameId]) => {
			if (
				refreshedProviderId !== game.providerId ||
				refreshedGameId !== game.gameId
			)
				return;
			updateInstalledModVersions();
		},
	);

	const filteredMods = useMemo(() => {
		return Object.values(mods).filter(
			(mod) =>
				game &&
				(!mod.common.engine || mod.common.engine === game.engineBrand) &&
				(!mod.common.unityBackend ||
					!game.unityBackend ||
					mod.common.unityBackend === game.unityBackend) &&
				isVersionWithinRange(game, mod.common.engineVersionRange) &&
				!(mod.remote?.deprecated && !installedModVersions[mod.common.id]),
		);
	}, [game, installedModVersions, mods]);

	if (filteredMods.length === 0) {
		return null;
	}

	return (
		<>
			<Divider label={t("gameModsLabel")} />
			{!game.exePath && (
				<Alert color="orange">{t("gameNotInstalledWarning")}</Alert>
			)}
			<TableContainer bg="dark">
				<Table>
					<Table.Tbody>
						{filteredMods.map((mod) => {
							const modLoader = modLoaderMap[mod.common.loaderId];

							return (
								modLoader && (
									<GameModRow
										key={mod.common.id}
										game={game}
										mod={mod}
										modLoader={modLoader}
										remoteConfigs={remoteConfigs}
										installedVersion={installedModVersions[mod.common.id]}
									/>
								)
							);
						})}
					</Table.Tbody>
				</Table>
			</TableContainer>
			{game.exePath && (
				<CommandButton
					confirmationText={t("uninstallAllModsConfirmation")}
					onClick={() =>
						commands.uninstallAllMods(game.providerId, game.gameId)
					}
					color="red"
					variant="light"
					leftSection={<IconTrash />}
				>
					{t("uninstallAllModsButton")}
				</CommandButton>
			)}
		</>
	);
}
