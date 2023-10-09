import { GameEngineBrand } from "@api/bindings";
import { Badge, BadgeProps } from "@mantine/core";
import { engineColor } from "../../util/color";

interface Props extends BadgeProps {
	readonly engine?: GameEngineBrand | null;
}

export function EngineBadge(props: Props) {
	return (
		<Badge
			color={props.engine ? engineColor[props.engine] : "dark"}
			{...props}
		>
			{props.engine ?? "Unknown"}{" "}
		</Badge>
	);
}
