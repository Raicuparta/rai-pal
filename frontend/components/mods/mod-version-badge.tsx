import { Badge, DefaultMantineColor, Stack, Tooltip } from "@mantine/core";
import { getIsOutdated, ModVersionInfo } from "@util/is-outdated";
import { IconRefreshAlert } from "@tabler/icons-react";
import { useLocalization } from "@hooks/use-localization";

type Props = {
	readonly current?: ModVersionInfo;
	readonly latest?: ModVersionInfo;
};

function getColor(props: Props): DefaultMantineColor {
	if (!props.current) return "gray";
	if (getIsOutdated(props.current, props.latest)) return "orange";
	return "green";
}

export function ModVersionBadge(props: Props) {
	const t = useLocalization("modsPage");
	const isOutdated = getIsOutdated(props.current, props.latest);

	if (!props.current && !props.latest) {
		return null;
	}

	const versionText = (
		props.current?.version ||
		props.latest?.version ||
		"-"
	).split("/")[0];

	return (
		<Tooltip
			disabled={!isOutdated}
			label={t("modOutdated")}
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
