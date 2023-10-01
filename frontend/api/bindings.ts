/* eslint-disable */
// This file was generated by [tauri-specta](https://github.com/oscartbeaumont/tauri-specta). Do not edit this file manually.

declare global {
    interface Window {
        __TAURI_INVOKE__<T>(cmd: string, args?: Record<string, unknown>): Promise<T>;
    }
}

// Function avoids 'window not defined' in SSR
const invoke = () => window.__TAURI_INVOKE__;

export function getGameMap(ignoreCache: boolean) {
    return invoke()<{ [key: string]: Game }>("get_game_map", { ignoreCache })
}

export function getOwnedGames(ignoreCache: boolean) {
    return invoke()<OwnedGame[]>("get_owned_games", { ignoreCache })
}

export function openGameFolder(gameId: string) {
    return invoke()<null>("open_game_folder", { gameId })
}

export function getModLoaders(ignoreCache: boolean) {
    return invoke()<{ [key: string]: ModLoaderData }>("get_mod_loaders", { ignoreCache })
}

export function installMod(modLoaderId: string, modId: string, gameId: string) {
    return invoke()<null>("install_mod", { modLoaderId,modId,gameId })
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

export function openModFolder(modLoaderId: string, modId: string) {
    return invoke()<null>("open_mod_folder", { modLoaderId,modId })
}

export function updateGameInfo(gameId: string) {
    return invoke()<{ [key: string]: Game }>("update_game_info", { gameId })
}

export function deleteSteamAppinfoCache() {
    return invoke()<null>("delete_steam_appinfo_cache")
}

export type GameEngine = { brand: GameEngineBrand; version: GameEngineVersion }
export type UnityScriptingBackend = "Il2Cpp" | "Mono" | "Unknown"
export type Mod = { id: string; name: string; scriptingBackend: UnityScriptingBackend; path: string }
export type GameEngineBrand = "Unity" | "Unreal" | "Godot" | "Unknown"
export type GameEngineVersion = { major: number; minor: number; patch: number; suffix: string; display: string }
export type SteamLaunchOption = { launchId: string; appId: number; description: string | null; executable: string | null; arguments: string | null; appType: string | null; osList: string | null; betaKey: string | null; osArch: string | null }
export type OperatingSystem = "Unknown" | "Linux" | "Windows"
export type Game = { id: string; name: string; discriminator: string | null; fullPath: string; architecture: Architecture; scriptingBackend: UnityScriptingBackend; operatingSystem: OperatingSystem; steamLaunch: SteamLaunchOption | null; installedMods: string[]; engine: GameEngine }
export type OwnedGame = { id: string; name: string; installed: boolean; osList: OperatingSystem[]; engine: GameEngineBrand; releaseDate: number }
export type ModLoaderData = { id: string; path: string; mods: Mod[] }
export type Architecture = "Unknown" | "X64" | "X86"
