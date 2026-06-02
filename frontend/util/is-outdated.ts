export type ModVersionInfo = {
	version: string;
	hash: string;
};

export function getIsOutdated(
	current?: ModVersionInfo,
	latest?: ModVersionInfo,
) {
	if (!current || !latest) return false;

	return current?.version !== latest?.version || current.hash !== latest.hash;
}
