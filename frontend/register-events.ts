export function registerEvents() {
	preventPrintDialog();
	preventContextMenu();
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
