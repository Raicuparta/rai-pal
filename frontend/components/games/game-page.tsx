import { Alert, Box, Button, Divider, Stack, Table } from "@mantine/core";
import { commands, DbGame, ProviderCommandAction } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import {
	IconFileSettings,
	IconFolder,
	IconFolderCog,
	IconGlassFull,
	IconRefresh,
} from "@tabler/icons-react";
import { DebugData } from "@components/debug-data";
import { GamesColgroup } from "./games-columns";
import { TableContainer } from "@components/table/table-container";
import { CommandDropdown } from "@components/command-dropdown";
import { GameRowInner } from "./game-row";
import { useLocalization } from "@hooks/use-localization";
import { useAsyncCommand } from "@hooks/use-async-command";
import { RemoveGameButton } from "./remove-game-button";
import { platform } from "@tauri-apps/plugin-os";
import { Page } from "@components/page";
import { GameModsData } from "@hooks/use-selected-game";
import { GameMods } from "./game-mods";
import { ProviderCommandButton } from "@components/providers/provider-command-button";

type Props = {
	readonly game: DbGame;
	readonly mods: GameModsData;
};

export function GamePage({ game, mods }: Props) {
	const { t } = useLocalization("gameModal");
	const [close] = useAsyncCommand(() => commands.setSelectedGame(null, null));

	const { providerId, gameId } = game;

	const {
		StartViaProvider: startViaProvider,
		StartViaExe: startViaExe,
		...otherProviderCommands
	} = game.providerCommands;

	const providerCommandActions = Object.keys(
		otherProviderCommands,
	) as ProviderCommandAction[];

	const [primaryStart, secondaryStart]: readonly ProviderCommandAction[] =
		startViaProvider
			? ["StartViaProvider", "StartViaExe"]
			: startViaExe
				? ["StartViaExe"]
				: [];

	return (
		<Page
			onClose={close}
			sidebar={
				<Stack
					px="xs"
					gap="lg"
				>
					{primaryStart && (
						<Button.Group>
							<ProviderCommandButton
								variant="filled"
								fullWidth
								game={game}
								action={primaryStart}
							/>

							{secondaryStart && (
								<CommandDropdown>
									<ProviderCommandButton
										game={game}
										action={secondaryStart}
									/>
								</CommandDropdown>
							)}
						</Button.Group>
					)}
					{game.exePath && (
						<CommandButton
							onClick={() => commands.refreshGame(providerId, gameId)}
							leftSection={<IconRefresh />}
						>
							{t("refreshGame")}
						</CommandButton>
					)}
					{providerId === "Manual" && (
						<RemoveGameButton
							providerId={providerId}
							gameId={gameId}
						/>
					)}
					{providerCommandActions.length > 0 && (
						<Stack>
							<Divider label={game.providerId} />
							<Button.Group orientation="vertical">
								{providerCommandActions.map((action) => (
									<ProviderCommandButton
										key={action}
										game={game}
										action={action}
									/>
								))}
							</Button.Group>
						</Stack>
					)}
					{game.exePath && (
						<Stack>
							<Divider label={t("foldersDropdown")} />
							<Button.Group orientation="vertical">
								<CommandButton
									leftSection={<IconFolder />}
									onClick={() => commands.openGameFolder(providerId, gameId)}
								>
									{t("openGameFilesFolder")}
								</CommandButton>
								<CommandButton
									leftSection={<IconFolderCog />}
									onClick={() =>
										commands.openGameModsFolder(providerId, gameId)
									}
								>
									{t("openInstalledModsFolder")}
								</CommandButton>
								<CommandButton
									leftSection={<IconFileSettings />}
									onClick={() =>
										commands.openGameDataFolder(providerId, gameId)
									}
								>
									{t("openGameDataFolder")}
								</CommandButton>
								{platform() === "linux" && (
									<>
										<CommandButton
											leftSection={<IconGlassFull />}
											onClick={() =>
												commands.openGameWinePrefixFolder(providerId, gameId)
											}
										>
											{t("openGameWinePrefixFolder")}
										</CommandButton>
										<CommandButton
											leftSection={<IconGlassFull />}
											onClick={() =>
												commands.openGameWineBinaryFolder(providerId, gameId)
											}
										>
											{t("openGameWineBinaryFolder")}
										</CommandButton>
									</>
								)}
							</Button.Group>
						</Stack>
					)}
				</Stack>
			}
		>
			<Box>
				<TableContainer>
					<Table highlightOnHover>
						<GamesColgroup />
						<Table.Tbody>
							<GameRowInner
								game={game}
								onClick={close}
							/>
						</Table.Tbody>
					</Table>
				</TableContainer>
			</Box>
			<Stack
				p="xs"
				gap="xl"
			>
				{game.exePath && (
					<>
						{game.engineBrand && !game.architecture && (
							<Alert color="red">{t("failedToReadGameInfo")}</Alert>
						)}
						{!game.engineBrand && (
							<Alert color="red">{t("failedToDetermineEngine")}</Alert>
						)}
					</>
				)}
				<GameMods
					game={game}
					mods={mods}
				/>
				<DebugData data={{ game, mods }} />
			</Stack>
		</Page>
	);
}
