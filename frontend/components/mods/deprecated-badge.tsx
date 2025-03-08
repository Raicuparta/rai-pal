import { useLocalization } from "@hooks/use-localization";
import { Badge, BadgeProps, Tooltip } from "@mantine/core";
import { IconAlertTriangleFilled } from "@tabler/icons-react";

export const DeprecatedBadge = (props: BadgeProps) => {
	const t = useLocalization("modsPage");
	return (
		<Tooltip label={t("modDeprecatedTooltip")}>
			<Badge
				color="yellow"
				leftSection={<IconAlertTriangleFilled />}
				{...props}
			>
				{t("modDeprecated")}
			</Badge>
		</Tooltip>
	);
};
