/* eslint-disable */
// This file was generated by [tauri-specta](https://github.com/oscartbeaumont/tauri-specta). Do not edit this file manually.

declare global {
    interface Window {
        __TAURI_INVOKE__<T>(cmd: string, args?: Record<string, unknown>): Promise<T>;
    }
}

// Function avoids 'window not defined' in SSR
const invoke = () => window.__TAURI_INVOKE__;

export function dummyCommand() {
    return invoke()<[InstalledGame, AppEvent]>("dummy_command")
}

export function updateData() {
    return invoke()<null>("update_data")
}

export function getInstalledGames() {
    return invoke()<{ [key: string]: InstalledGame }>("get_installed_games")
}

export function getOwnedGames() {
    return invoke()<{ [key: string]: OwnedGame }>("get_owned_games")
}

export function getModLoaders() {
    return invoke()<{ [key: string]: ModLoaderData }>("get_mod_loaders")
}

export function openGameFolder(gameId: string) {
    return invoke()<null>("open_game_folder", { gameId })
}

export function installMod(gameId: string, modId: string) {
    return invoke()<null>("install_mod", { gameId,modId })
}

export function uninstallMod(gameId: string, modId: string) {
    return invoke()<null>("uninstall_mod", { gameId,modId })
}

export function openGameModsFolder(gameId: string) {
    return invoke()<null>("open_game_mods_folder", { gameId })
}

export function startGame(gameId: string) {
    return invoke()<null>("start_game", { gameId })
}

export function startGameExe(gameId: string) {
    return invoke()<null>("start_game_exe", { gameId })
}

export function openModFolder(modId: string) {
    return invoke()<null>("open_mod_folder", { modId })
}

export function downloadMod(modId: string) {
    return invoke()<null>("download_mod", { modId })
}

export function openModsFolder() {
    return invoke()<null>("open_mods_folder")
}

export function addGame(path: string) {
    return invoke()<null>("add_game", { path })
}

export function removeGame(gameId: string) {
    return invoke()<null>("remove_game", { gameId })
}

export function deleteSteamAppinfoCache() {
    return invoke()<null>("delete_steam_appinfo_cache")
}

export function frontendReady() {
    return invoke()<null>("frontend_ready")
}

export function getLocalMods() {
    return invoke()<{ [key: string]: LocalMod }>("get_local_mods")
}

export function getRemoteMods() {
    return invoke()<{ [key: string]: RemoteMod }>("get_remote_mods")
}

export function openModLoaderFolder(modLoaderId: string) {
    return invoke()<null>("open_mod_loader_folder", { modLoaderId })
}

export function refreshGame(gameId: string) {
    return invoke()<null>("refresh_game", { gameId })
}

export function openLogsFolder() {
    return invoke()<null>("open_logs_folder")
}

export type GameEngineVersion = { major: number; minor: number; patch: number; suffix: string | null; display: string }
export type GameExecutable = { path: string; engine: GameEngine | null; architecture: Architecture | null; operatingSystem: OperatingSystem | null; scriptingBackend: UnityScriptingBackend | null }
export type UnityScriptingBackend = "Il2Cpp" | "Mono"
export type GameEngineBrand = "Unity" | "Unreal" | "Godot"
export type GameMode = "VR" | "Flat"
export type InstalledGame = { id: string; name: string; providerId: ProviderId; discriminator: string | null; steamLaunch: SteamLaunchOption | null; executable: GameExecutable; thumbnailUrl: string | null; installedModVersions: { [key: string]: string | null }; gameMode: GameMode }
export type OwnedGame = { id: string; providerId: ProviderId; name: string; installed: boolean; osList: OperatingSystem[]; engine: GameEngineBrand | null; releaseDate: number; thumbnailUrl: string; gameMode: GameMode | null; uevrScore: UevrScore | null }
export type AppEvent = "SyncInstalledGames" | "SyncOwnedGames" | "SyncModLoaders" | "SyncLocalMods" | "SyncRemoteMods" | "ExecutedSteamCommand" | "GameAdded" | "GameRemoved" | "Error"
export type Architecture = "X64" | "X86"
export type Manifest = { version: string }
export type RemoteModData = { title: string; author: string; sourceCode: string; description: string; latestVersion: ModDownload | null }
export type SteamLaunchOption = { launchId: string; appId: number; description: string | null; executable: string | null; arguments: string | null; appType: string | null; osList: string | null; betaKey: string | null; osArch: string | null }
export type OperatingSystem = "Linux" | "Windows"
export type LocalModData = { path: string; manifest: Manifest | null }
export type LocalMod = { data: LocalModData; common: CommonModData }
export type RemoteMod = { common: CommonModData; data: RemoteModData }
export type ModLoaderData = { id: string; path: string; kind: ModKind }
export type ModDownload = { id: string; url: string; root: string | null }
export type ProviderId = "Steam" | "Manual" | "Epic" | "Gog" | "Xbox"
export type GameEngine = { brand: GameEngineBrand; version: GameEngineVersion | null }
export type CommonModData = { id: string; engine: GameEngineBrand | null; unityBackend: UnityScriptingBackend | null; loaderId: string }
export type ModKind = "Installable" | "Runnable"
export type UevrScore = "A" | "B" | "C" | "D" | "E"
