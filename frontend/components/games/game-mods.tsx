import { Alert, Box, Stack, Table } from "@mantine/core";
import { DbGame, ModFamily, commands } from "@api/bindings";
import { ReactNode, useCallback, useMemo } from "react";
import { useAtomValue } from "jotai";
import { CommandButton } from "@components/command-button";
import { IconTrash } from "@tabler/icons-react";
import { GameModRow } from "./game-mod-row";
import { useLocalization } from "@hooks/use-localization";
import { useCommandData } from "@hooks/use-command-data";
import { GameModsData, GameModsPart } from "@hooks/use-selected-game";
import { modFamiliesAtom } from "@hooks/use-data";
import { ModsSection } from "@components/mods/mods-section";
import { ModSummary } from "@components/mods/mod-summary";

type PotentialMod =
	| { readonly type: "mod"; readonly mod: GameModsPart["mod"] }
	| { readonly type: "family"; readonly family: ModFamily };

type Props = {
	readonly game: DbGame;
	readonly mods: GameModsData;
};

function GameModsSection({
	game,
	mods,
	remoteConfigModIds,
	incompatible,
	header,
	label,
	description,
}: {
	readonly game: DbGame;
	readonly mods: GameModsPart[];
	readonly remoteConfigModIds: ReadonlySet<string>;
	readonly label: string;
	readonly description?: string;
	readonly incompatible?: boolean;
	readonly header?: ReactNode;
}) {
	return (
		<ModsSection
			label={label}
			description={description}
		>
			{header}
			{mods.map(({ mod, info }) => (
				<GameModRow
					key={mod.id}
					game={game}
					mod={mod}
					info={info}
					hasRemoteConfig={remoteConfigModIds.has(mod.id)}
					incompatible={incompatible}
				/>
			))}
		</ModsSection>
	);
}

function PotentialModsSection({
	label,
	description,
	mods,
}: {
	readonly label: string;
	readonly description?: string;
	readonly mods: PotentialMod[];
}) {
	return (
		<ModsSection
			label={label}
			description={description}
		>
			{mods.map((entry) => {
				const isMod = entry.type === "mod";
				const title = isMod ? entry.mod.title : entry.family.title;
				const entryDescription = isMod
					? entry.mod.description
					: entry.family.description;

				return (
					<Table.Tr key={isMod ? entry.mod.id : title}>
						<Table.Td>
							<ModSummary
								title={title}
								description={entryDescription}
							/>
						</Table.Td>
					</Table.Tr>
				);
			})}
		</ModsSection>
	);
}

export function GameMods({ game, mods }: Props) {
	const { t } = useLocalization("gameModal");
	const modFamilies = useAtomValue(modFamiliesAtom);
	const getRemoteConfigModIds = useCallback(
		() => commands.getRemoteConfigModIds(game.providerId, game.gameId),
		[game],
	);
	const [remoteConfigModIds] = useCommandData(getRemoteConfigModIds, []);
	const remoteConfigModIdSet = new Set(remoteConfigModIds);

	const potentialMods = useMemo<PotentialMod[]>(() => {
		const familyMods = new Map<string, GameModsPart[]>();
		const result: PotentialMod[] = [];

		for (const part of mods.compatibleMods) {
			if (part.mod.family) {
				const existing = familyMods.get(part.mod.family);
				if (existing) {
					existing.push(part);
				} else {
					familyMods.set(part.mod.family, [part]);
				}
			} else {
				result.push({ type: "mod", mod: part.mod });
			}
		}

		for (const [familyId, familyModsList] of familyMods) {
			const family = modFamilies[familyId];
			if (familyModsList.length > 1 && family) {
				result.push({ type: "family", family });
			} else {
				for (const part of familyModsList) {
					result.push({ type: "mod", mod: part.mod });
				}
			}
		}

		return result;
	}, [mods.compatibleMods, modFamilies]);

	if (mods.compatibleMods.length + mods.incompatibleMods.length === 0) {
		return (
			<Alert
				color="blue"
				m="md"
			>
				{t("noModsForGame")}
			</Alert>
		);
	}

	if (!game.exePath) {
		return (
			<PotentialModsSection
				label={t("potentiallyCompatibleMods")}
				description={t("potentiallyCompatibleModsDescription")}
				mods={potentialMods}
			/>
		);
	}

	const installedMods = mods.compatibleMods.filter(
		({ info }) => info.installedHash,
	);
	const notInstalledMods = mods.compatibleMods.filter(
		({ info }) => !info.installedHash,
	);

	return (
		<>
			<Stack gap="lg">
				{mods.compatibleMods.length > 0 && (
					<>
						{installedMods.length > 0 && (
							<Stack>
								<GameModsSection
									label={t("installedMods")}
									game={game}
									mods={installedMods}
									remoteConfigModIds={remoteConfigModIdSet}
								/>
								<Box px="xs">
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
								</Box>
							</Stack>
						)}
						{notInstalledMods.length > 0 && (
							<GameModsSection
								label={t("availableMods")}
								game={game}
								mods={notInstalledMods}
								remoteConfigModIds={remoteConfigModIdSet}
							/>
						)}
						{mods.hiddenMods.length > 0 && (
							<GameModsSection
								label={t("otherThings")}
								description={t("otherThingsDescription")}
								game={game}
								mods={mods.hiddenMods}
								remoteConfigModIds={remoteConfigModIdSet}
							/>
						)}
					</>
				)}
			</Stack>
			{mods.incompatibleMods.length > 0 && (
				<GameModsSection
					label={t("incompatibleGameModsLabel")}
					description={t("incompatibleGameModsDescription")}
					game={game}
					mods={mods.incompatibleMods}
					remoteConfigModIds={remoteConfigModIdSet}
					incompatible
				/>
			)}
		</>
	);
}
