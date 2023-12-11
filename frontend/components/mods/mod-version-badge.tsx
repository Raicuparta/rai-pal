import {
	Badge,
	DefaultMantineColor,
	Stack,
	ThemeIcon,
	Tooltip,
} from "@mantine/core";
import { IconAlertTriangleFilled } from "@tabler/icons-react";
import { isOutdated } from "../../util/is-outdated";

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
