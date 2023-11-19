import { addGame } from "@api/bindings";
import { event } from "@tauri-apps/api";

export function registerEvents() {
	preventPrintDialog();
	preventContextMenu();
	listenToFileDrop();
}

function preventPrintDialog() {
	document.addEventListener("keydown", (e) => {
		if (e.ctrlKey && e.key === "p") {
			e.preventDefault();
		}
	});
}

function preventContextMenu() {
	if (import.meta.env.PROD) {
		document.addEventListener("contextmenu", (e) => {
			e.preventDefault();
		});
	}
}

function listenToFileDrop() {
	event.listen<string[]>(event.TauriEvent.WINDOW_FILE_DROP, (event) => {
		if (event.payload.length > 0) {
			addGame(event.payload[0]);
		}
	});
}
