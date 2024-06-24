import { invoke } from "@tauri-apps/api/core";

export function dummyCommand() {
	return invoke<[InstalledGame, AppEvent, ProviderCommandAction]>(
		"dummy_command",
	);
}

export function updateData() {
	return invoke<null>("update_data");
}

export function getInstalledGames() {
	return invoke<{ [key: string]: InstalledGame }>("get_installed_games");
}

export function getOwnedGames() {
	return invoke<{ [key: string]: OwnedGame }>("get_owned_games");
}

export function getModLoaders() {
	return invoke<{ [key: string]: ModLoaderData }>("get_mod_loaders");
}

export function openGameFolder(gameId: string) {
	return invoke<null>("open_game_folder", { gameId });
}

export function installMod(gameId: string, modId: string) {
	return invoke<null>("install_mod", { gameId, modId });
}

export function configureMod(gameId: string, modId: string) {
	return invoke<null>("configure_mod", { gameId, modId });
}

export function openInstalledModFolder(gameId: string, modId: string) {
	return invoke<null>("open_installed_mod_folder", { gameId, modId });
}

export function uninstallMod(gameId: string, modId: string) {
	return invoke<null>("uninstall_mod", { gameId, modId });
}

export function uninstallAllMods(gameId: string) {
	return invoke<null>("uninstall_all_mods", { gameId });
}

export function openGameModsFolder(gameId: string) {
	return invoke<null>("open_game_mods_folder", { gameId });
}

export function startGame(gameId: string) {
	return invoke<null>("start_game", { gameId });
}

export function startGameExe(gameId: string) {
	return invoke<null>("start_game_exe", { gameId });
}

export function openModFolder(modId: string) {
	return invoke<null>("open_mod_folder", { modId });
}

export function downloadMod(modId: string) {
	return invoke<null>("download_mod", { modId });
}

export function runRunnableWithoutGame(modId: string) {
	return invoke<null>("run_runnable_without_game", { modId });
}

export function deleteMod(modId: string) {
	return invoke<null>("delete_mod", { modId });
}

export function openModsFolder() {
	return invoke<null>("open_mods_folder");
}

export function addGame(path: string) {
	return invoke<null>("add_game", { path });
}

export function removeGame(gameId: string) {
	return invoke<null>("remove_game", { gameId });
}

export function deleteSteamAppinfoCache() {
	return invoke<null>("delete_steam_appinfo_cache");
}

export function frontendReady() {
	return invoke<null>("frontend_ready");
}

export function getLocalMods() {
	return invoke<{ [key: string]: LocalMod }>("get_local_mods");
}

export function getRemoteMods() {
	return invoke<{ [key: string]: RemoteMod }>("get_remote_mods");
}

export function getRemoteGames() {
	return invoke<{ [key: string]: RemoteGame }>("get_remote_games");
}

export function openModLoaderFolder(modLoaderId: string) {
	return invoke<null>("open_mod_loader_folder", { modLoaderId });
}

export function refreshGame(gameId: string) {
	return invoke<null>("refresh_game", { gameId });
}

export function openLogsFolder() {
	return invoke<null>("open_logs_folder");
}

export function runProviderCommand(ownedGameId: string, commandAction: string) {
	return invoke<null>("run_provider_command", { ownedGameId, commandAction });
}

export type ModDownload = {
	id: string;
	url: string;
	root: string | null;
	runnable: RunnableModData | null;
};
export type UnityScriptingBackend = "Il2Cpp" | "Mono";
export type RemoteMod = { common: CommonModData; data: RemoteModData };
export type GameMode = "VR" | "Flat";
export type EngineVersion = {
	numbers: EngineVersionNumbers;
	suffix: string | null;
	display: string;
};
export type EngineVersionRange = {
	minimum: EngineVersionNumbers | null;
	maximum: EngineVersionNumbers | null;
};
export type ModKind = "Installable" | "Runnable";
export type GameEngine = { brand: EngineBrand; version: EngineVersion | null };
export type AppEvent =
	| "SyncInstalledGames"
	| "SyncOwnedGames"
	| "SyncRemoteGames"
	| "SyncModLoaders"
	| "SyncLocalMods"
	| "SyncRemoteMods"
	| "ExecutedProviderCommand"
	| "GameAdded"
	| "GameRemoved"
	| "Error";
export type Manifest = {
	title: string | null;
	version: string;
	runnable: RunnableModData | null;
	engine: EngineBrand | null;
	engineVersionRange: EngineVersionRange | null;
	unityBackend: UnityScriptingBackend | null;
};
export type LocalMod = { data: LocalModData; common: CommonModData };
export type InstalledGame = {
	id: string;
	name: string;
	provider: ProviderId;
	executable: GameExecutable;
	installedModVersions: { [key: string]: string };
	discriminator: string | null;
	thumbnailUrl: string | null;
	ownedGameId: string | null;
	startCommand: ProviderCommand | null;
};
export type RunnableModData = { path: string; args: string[] };
export type RemoteGame = {
	id: string;
	engine: GameEngine | null;
	skipCache: boolean;
};
export type RemoteModData = {
	title: string;
	deprecated: boolean;
	author: string;
	sourceCode: string;
	description: string;
	latestVersion: ModDownload | null;
};
export type LocalModData = { path: string; manifest: Manifest | null };
export type ModLoaderData = { id: string; path: string; kind: ModKind };
export type GameExecutable = {
	path: string;
	name: string;
	engine: GameEngine | null;
	architecture: Architecture | null;
	operatingSystem: OperatingSystem | null;
	scriptingBackend: UnityScriptingBackend | null;
};
export type ProviderCommandAction =
	| "Install"
	| "ShowInLibrary"
	| "ShowInStore"
	| "Start"
	| "OpenInBrowser";
export type ProviderId = "Steam" | "Manual" | "Itch" | "Epic" | "Gog" | "Xbox";
export type EngineBrand = "Unity" | "Unreal" | "Godot" | "GameMaker";
export type EngineVersionNumbers = {
	major: number;
	minor: number | null;
	patch: number | null;
};
export type AppType = "Game" | "Demo";
export type OperatingSystem = "Linux" | "Windows";
export type ProviderCommand = { String: string } | { Path: [string, string[]] };
export type Architecture = "X64" | "X86";
export type CommonModData = {
	id: string;
	engine: EngineBrand | null;
	unityBackend: UnityScriptingBackend | null;
	engineVersionRange: EngineVersionRange | null;
	loaderId: string;
};
export type OwnedGame = {
	id: string;
	provider: ProviderId;
	name: string;
	osList: OperatingSystem[];
	releaseDate: bigint | null;
	thumbnailUrl: string | null;
	gameMode: GameMode | null;
	appType: AppType | null;
	providerCommands: { [key: string]: ProviderCommand };
};
