import { shell } from "@tauri-apps/api";

function runSteamCommand(command: string) {
	return async function (appId?: string | number) {
		if (!appId) return;

		return shell.open(`steam://${command}/${appId}`);
	};
}

export const steamCommands = {
	showInLibrary: runSteamCommand("nav/games/details"),
	openStorePage: runSteamCommand("store"),
	install: runSteamCommand("install"),
};
