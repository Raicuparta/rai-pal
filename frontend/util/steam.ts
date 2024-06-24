import { showAppNotification } from "@components/app-notifications";
import { open as shellOpen } from "@tauri-apps/plugin-shell";

function runSteamCommand(command: string) {
	return async function (appId?: string | number) {
		if (!appId) return;

		showAppNotification("Running Steam command...", "info");

		return shellOpen(`steam://${command}/${appId}`);
	};
}

export const steamCommands = {
	showInLibrary: runSteamCommand("nav/games/details"),
	openStorePage: runSteamCommand("store"),
	install: runSteamCommand("install"),
};
