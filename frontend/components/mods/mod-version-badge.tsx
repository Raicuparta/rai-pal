import { Badge, DefaultMantineColor, Stack, Tooltip } from "@mantine/core";
import { getIsOutdated } from "@util/is-outdated";
import { IconRefreshAlert } from "@tabler/icons-react";
import { useLocalization } from "@hooks/use-localization";
import { GameMod } from "@api/bindings";

type Props = {
	readonly local?: GameMod;
	readonly remote?: GameMod;
};

function getColor(props: Props): DefaultMantineColor {
	if (!props.local) return "gray";
	if (getIsOutdated(props.local, props.remote)) return "orange";
	return "green";
}

export function ModVersionBadge(props: Props) {
	const t = useLocalization("modsPage");
	const isOutdated = getIsOutdated(props.local, props.remote);

	if (!props.local?.download && !props.remote?.download) {
		return null;
	}

	const versionText = (
		props.local?.download?.id ||
		props.remote?.download?.id ||
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
