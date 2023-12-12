export function isOutdated(
	localVersion: string | undefined | null,
	remoteVersion: string | undefined | null,
) {
	return (
		Boolean(localVersion && remoteVersion) && localVersion !== remoteVersion
	);
}
