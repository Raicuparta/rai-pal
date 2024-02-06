import { Badge, DefaultMantineColor, Stack, Tooltip } from "@mantine/core";
import { getIsOutdated } from "../../util/is-outdated";
import { OutdatedMarker } from "@components/outdated-marker";

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
	return (
		<Tooltip
			disabled={!isOutdated}
			label="Mod outdated. Re-download it to update."
		>
			<Stack
				gap={5}
				align="center"
			>
				<Badge
					color={getColor(props)}
					maw={150}
				>
					{props.localVersion || props.remoteVersion || "-"}
				</Badge>
				{isOutdated && <OutdatedMarker />}
			</Stack>
		</Tooltip>
	);
}
