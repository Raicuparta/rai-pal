import { Badge, BadgeProps, Tooltip } from "@mantine/core";
import { IconAlertTriangleFilled } from "@tabler/icons-react";

export const DeprecatedBadge = (props: BadgeProps) => (
	<Tooltip label="This mod is deprecated. You should uninstall it and install a newer alternative.">
		<Badge
			color="yellow"
			leftSection={<IconAlertTriangleFilled />}
			{...props}
		>
			Deprecated
		</Badge>
	</Tooltip>
);
