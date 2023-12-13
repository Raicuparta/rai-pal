import { Badge, DefaultMantineColor, Stack, Tooltip } from "@mantine/core";
import { isOutdated } from "../../util/is-outdated";
import { OutdatedMarker } from "@components/OutdatedMarker";

type Props = {
	readonly localVersion?: string;
	readonly remoteVersion?: string;
};

function getColor(props: Props): DefaultMantineColor {
	if (!props.localVersion) return "gray";
	if (isOutdated(props.localVersion, props.remoteVersion)) return "orange";
	return "green";
}

export function ModVersionBadge(props: Props) {
	const outdated = isOutdated(props.localVersion, props.remoteVersion);
	return (
		<Tooltip
			disabled={!outdated}
			label="Mod outdated. Re-download it to update."
		>
			<Stack
				gap={5}
				align="center"
			>
				<Badge color={getColor(props)}>
					{props.localVersion || props.remoteVersion || "-"}
				</Badge>
				{outdated && <OutdatedMarker />}
			</Stack>
		</Tooltip>
	);
}
