import { useGetTranslated } from "@hooks/use-translations";
import { Badge, BadgeProps, Tooltip } from "@mantine/core";
import { IconAlertTriangleFilled } from "@tabler/icons-react";

export const DeprecatedBadge = (props: BadgeProps) => {
	const t = useGetTranslated("modsPage");
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
