import { Badge, DefaultMantineColor, Stack, Tooltip } from "@mantine/core";
import { getIsOutdated } from "@util/is-outdated";
import { IconRefreshAlert } from "@tabler/icons-react";

type Props = {
	readonly localVersion?: string;
	readonly remoteVersion?: string;
};

function getColor(props: Props): DefaultMantineColor {
	if (!props.localVersion) return "gray";
	if (getIsOutdated(props.localVersion, props.remoteVersion)) return "orange";
	return "green";
}

export function ModVersionBadge(props: Props) {
	const isOutdated = getIsOutdated(props.localVersion, props.remoteVersion);

	const versionText = (props.localVersion || props.remoteVersion || "-").split(
		"/",
	)[0];

	return (
		<Tooltip
			disabled={!isOutdated}
			label="Mod outdated"
		>
			<Stack
				gap={5}
				align="center"
			>
				<Badge
					color={getColor(props)}
					maw={150}
					leftSection={isOutdated && <IconRefreshAlert fontSize={15} />}
				>
					{versionText}
				</Badge>
			</Stack>
		</Tooltip>
	);
}
