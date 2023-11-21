export function registerEvents() {
	preventPrintDialog();
	preventContextMenu();
	preventFindCtrlF();
}

function preventPrintDialog() {
	document.addEventListener("keydown", (e) => {
		if (e.ctrlKey && e.key === "p") {
			e.preventDefault();
		}
	});
}

function preventFindCtrlF() {
	document.addEventListener("keydown", (e) => {
		if (e.ctrlKey && e.key === "f") {
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
