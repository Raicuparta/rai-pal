
// This file was generated by [tauri-specta](https://github.com/oscartbeaumont/tauri-specta). Do not edit this file manually.

/** user-defined commands **/


export const commands = {
async addGame(path: string) : Promise<Result<null, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("add_game", { path }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async clearCache() : Promise<Result<null, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("clear_cache") };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async configureMod(gameId: GameId, modId: string) : Promise<Result<null, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("configure_mod", { gameId, modId }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async deleteMod(modId: string) : Promise<Result<null, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("delete_mod", { modId }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async downloadMod(modId: string) : Promise<Result<null, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("download_mod", { modId }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async frontendReady() : Promise<Result<null, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("frontend_ready") };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async getAppSettings() : Promise<Result<AppSettings, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("get_app_settings") };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async getGameIds(dataQuery: GamesQuery | null) : Promise<Result<GameIdsResponse, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("get_game_ids", { dataQuery }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async getGame(id: GameId) : Promise<Result<Game, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("get_game", { id }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async getLocalMods() : Promise<Result<Partial<{ [key in string]: LocalMod }>, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("get_local_mods") };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async getProviderIds() : Promise<Result<ProviderId[], Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("get_provider_ids") };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async getRemoteMods() : Promise<Result<Partial<{ [key in string]: RemoteMod }>, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("get_remote_mods") };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async installMod(gameId: GameId, modId: string) : Promise<Result<null, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("install_mod", { gameId, modId }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async openGameFolder(gameId: GameId) : Promise<Result<null, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("open_game_folder", { gameId }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async openGameModsFolder(gameId: GameId) : Promise<Result<null, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("open_game_mods_folder", { gameId }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async openInstalledModFolder(gameId: GameId, modId: string) : Promise<Result<null, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("open_installed_mod_folder", { gameId, modId }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async openLogsFolder() : Promise<Result<null, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("open_logs_folder") };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async openModFolder(modId: string) : Promise<Result<null, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("open_mod_folder", { modId }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async openModLoaderFolder(modLoaderId: string) : Promise<Result<null, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("open_mod_loader_folder", { modLoaderId }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async openModsFolder() : Promise<Result<null, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("open_mods_folder") };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async refreshGame(gameId: GameId) : Promise<Result<null, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("refresh_game", { gameId }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async refreshGames(providerId: ProviderId) : Promise<Result<null, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("refresh_games", { providerId }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async refreshMods() : Promise<Result<null, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("refresh_mods") };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async refreshRemoteGames() : Promise<Result<null, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("refresh_remote_games") };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async removeGame(path: string) : Promise<Result<null, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("remove_game", { path }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async resetSteamCache() : Promise<Result<null, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("reset_steam_cache") };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async runProviderCommand(game: Game, commandAction: ProviderCommandAction) : Promise<Result<null, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("run_provider_command", { game, commandAction }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async runRunnableWithoutGame(modId: string) : Promise<Result<null, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("run_runnable_without_game", { modId }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async saveAppSettings(settings: AppSettings) : Promise<Result<null, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("save_app_settings", { settings }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async startGameExe(installedGame: InstalledGame) : Promise<Result<null, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("start_game_exe", { installedGame }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async startGame(installedGame: InstalledGame) : Promise<Result<null, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("start_game", { installedGame }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async uninstallAllMods(gameId: GameId) : Promise<Result<null, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("uninstall_all_mods", { gameId }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async uninstallMod(gameId: GameId, modId: string) : Promise<Result<null, Error>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("uninstall_mod", { gameId, modId }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
}
}

/** user-defined events **/


export const events = __makeEvents__<{
errorRaised: ErrorRaised,
executedProviderCommand: ExecutedProviderCommand,
foundGame: FoundGame,
gamesChanged: GamesChanged,
selectInstalledGame: SelectInstalledGame,
syncLocalMods: SyncLocalMods,
syncModLoaders: SyncModLoaders,
syncRemoteMods: SyncRemoteMods
}>({
errorRaised: "error-raised",
executedProviderCommand: "executed-provider-command",
foundGame: "found-game",
gamesChanged: "games-changed",
selectInstalledGame: "select-installed-game",
syncLocalMods: "sync-local-mods",
syncModLoaders: "sync-mod-loaders",
syncRemoteMods: "sync-remote-mods"
})

/** user-defined constants **/



/** user-defined types **/

export type AppLocale = "EnUs" | "EsEs" | "FrFr" | "DeDe" | "PtPt" | "ZhCn" | "JaJp" | "KoKr"
export type AppSettings = { hideGameThumbnails: boolean; overrideLanguage: AppLocale | null }
export type Architecture = "X64" | "X86"
export type CommonModData = { id: string; engine: EngineBrand | null; unityBackend: UnityScriptingBackend | null; engineVersionRange: EngineVersionRange | null; loaderId: string }
export type EngineBrand = "Unity" | "Unreal" | "Godot" | "GameMaker"
export type EngineVersion = { numbers: EngineVersionNumbers; suffix: string | null; display: string }
export type EngineVersionNumbers = { major: number; minor: number | null; patch: number | null }
export type EngineVersionRange = { minimum: EngineVersionNumbers | null; maximum: EngineVersionNumbers | null }
export type Error = "Tauri" | "Core" | "Io" | "SerdeJson" | { FailedToGetResourcesPath: string } | { FailedToAccessStateData: string }
export type ErrorRaised = string
export type ExecutedProviderCommand = null
export type FoundGame = GameId
export type Game = { id: GameId; externalId: string; tags: GameTag[]; installedGame: InstalledGame | null; remoteGame: RemoteGame | null; title: GameTitle; thumbnailUrl: string | null; releaseDate: bigint | null; providerCommands: Partial<{ [key in ProviderCommandAction]: ProviderCommand }>; fromSubscriptions: GameSubscription[] }
export type GameEngine = { brand: EngineBrand; version: EngineVersion | null }
export type GameExecutable = { path: string; name: string; engine: GameEngine | null; architecture: Architecture | null; scriptingBackend: UnityScriptingBackend | null }
export type GameId = { providerId: ProviderId; gameId: string }
export type GameIdsResponse = { gameIds: GameId[]; totalCount: bigint }
export type GameSubscription = "UbisoftClassics" | "UbisoftPremium" | "XboxGamePass" | "EaPlay"
export type GameTag = "VR" | "Demo"
export type GameTitle = { display: string; normalized: string[] }
export type GamesChanged = []
export type GamesFilter = { providers: (ProviderId | null)[]; tags: (GameTag | null)[]; architectures: (Architecture | null)[]; unityScriptingBackends: (UnityScriptingBackend | null)[]; engines: (EngineBrand | null)[]; installed: (InstallState | null)[] }
export type GamesQuery = { filter: GamesFilter; search: string; sortBy: GamesSortBy; sortDescending: boolean }
export type GamesSortBy = "Title" | "Engine" | "ReleaseDate"
export type InstallState = "Installed" | "NotInstalled"
export type InstalledGame = { id: string; executable: GameExecutable; installedModVersions: Partial<{ [key in string]: string }>; discriminator: string | null; startCommand: ProviderCommand | null }
export type LocalMod = { data: LocalModData; common: CommonModData }
export type LocalModData = { path: string; manifest: Manifest | null }
export type Manifest = { title: string | null; version: string; runnable: RunnableModData | null; engine: EngineBrand | null; engineVersionRange: EngineVersionRange | null; unityBackend: UnityScriptingBackend | null }
export type ModDownload = { id: string; url: string; root: string | null; runnable: RunnableModData | null }
export type ModKind = "Installable" | "Runnable"
export type ModLoaderData = { id: string; path: string; kind: ModKind }
export type ProviderCommand = { String: string } | { Path: [string, string[]] }
export type ProviderCommandAction = "Install" | "ShowInLibrary" | "ShowInStore" | "Start" | "OpenInBrowser"
export type ProviderId = "Ea" | "Epic" | "Gog" | "Itch" | "Manual" | "Steam" | "Ubisoft" | "Xbox"
export type RemoteGame = { title: string | null; engine: GameEngine | null; ids: Partial<{ [key in ProviderId]: string[] }>; subscriptions: GameSubscription[] | null }
export type RemoteMod = { common: CommonModData; data: RemoteModData }
export type RemoteModData = { title: string; deprecated: boolean; author: string; sourceCode: string; description: string; latestVersion: ModDownload | null }
export type RunnableModData = { path: string; args: string[] }
export type SelectInstalledGame = [ProviderId, string]
export type SyncLocalMods = Partial<{ [key in string]: LocalMod }>
export type SyncModLoaders = Partial<{ [key in string]: ModLoaderData }>
export type SyncRemoteMods = Partial<{ [key in string]: RemoteMod }>
export type UnityScriptingBackend = "Il2Cpp" | "Mono"

/** tauri-specta globals **/

import {
	invoke as TAURI_INVOKE,
	Channel as TAURI_CHANNEL,
} from "@tauri-apps/api/core";
import * as TAURI_API_EVENT from "@tauri-apps/api/event";
import { type WebviewWindow as __WebviewWindow__ } from "@tauri-apps/api/webviewWindow";

type __EventObj__<T> = {
	listen: (
		cb: TAURI_API_EVENT.EventCallback<T>,
	) => ReturnType<typeof TAURI_API_EVENT.listen<T>>;
	once: (
		cb: TAURI_API_EVENT.EventCallback<T>,
	) => ReturnType<typeof TAURI_API_EVENT.once<T>>;
	emit: null extends T
		? (payload?: T) => ReturnType<typeof TAURI_API_EVENT.emit>
		: (payload: T) => ReturnType<typeof TAURI_API_EVENT.emit>;
};

export type Result<T, E> =
	| { status: "ok"; data: T }
	| { status: "error"; error: E };

function __makeEvents__<T extends Record<string, any>>(
	mappings: Record<keyof T, string>,
) {
	return new Proxy(
		{} as unknown as {
			[K in keyof T]: __EventObj__<T[K]> & {
				(handle: __WebviewWindow__): __EventObj__<T[K]>;
			};
		},
		{
			get: (_, event) => {
				const name = mappings[event as keyof T];

				return new Proxy((() => {}) as any, {
					apply: (_, __, [window]: [__WebviewWindow__]) => ({
						listen: (arg: any) => window.listen(name, arg),
						once: (arg: any) => window.once(name, arg),
						emit: (arg: any) => window.emit(name, arg),
					}),
					get: (_, command: keyof __EventObj__<any>) => {
						switch (command) {
							case "listen":
								return (arg: any) => TAURI_API_EVENT.listen(name, arg);
							case "once":
								return (arg: any) => TAURI_API_EVENT.once(name, arg);
							case "emit":
								return (arg: any) => TAURI_API_EVENT.emit(name, arg);
						}
					},
				});
			},
		},
	);
}
