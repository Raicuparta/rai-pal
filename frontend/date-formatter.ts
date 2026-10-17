export const dateFormatter = Intl.DateTimeFormat("default", {
	year: "numeric",
	month: "long",
	day: "2-digit",
});

export function compareReleaseDates(
	a: string | null | undefined,
	b: string | null | undefined,
) {
	if (!a) return b ? 1 : 0;
	if (!b) return -1;
	return b.localeCompare(a);
}
