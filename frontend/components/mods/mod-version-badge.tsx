import {
	Badge,
	DefaultMantineColor,
	Stack,
	ThemeIcon,
	Tooltip,
} from "@mantine/core";
import { IconAlertTriangleFilled } from "@tabler/icons-react";

type Props = {
	readonly localVersion?: string;
	readonly remoteVersion?: string;
};

function isOutdated(props: Props) {
	return props.localVersion && props.localVersion !== props.remoteVersion;
}

function getColor(props: Props): DefaultMantineColor {
	if (!props.localVersion) return "gray";
	if (isOutdated(props)) return "orange";
	return "green";
}

export function ModVersionBadge(props: Props) {
	const outdated = isOutdated(props);
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
				{outdated && (
					<ThemeIcon
						color="orange"
						radius="xl"
					>
						<IconAlertTriangleFilled fontSize={15} />
					</ThemeIcon>
				)}
			</Stack>
		</Tooltip>
	);
}
