export function preventEvents() {
	// Prevent ctrl+p from opening the print dialog
	document.addEventListener("keydown", (e) => {
		if (e.ctrlKey && e.key === "p") {
			e.preventDefault();
		}
	});

	// Prevent opening context menu
	if (import.meta.env.PROD) {
		document.addEventListener("contextmenu", (e) => {
			e.preventDefault();
		});
	}
}
