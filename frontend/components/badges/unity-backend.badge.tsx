import { UnityScriptingBackend } from "@api/bindings";
import { Badge, BadgeProps } from "@mantine/core";
import { scriptingBackendColor } from "../../util/color";

interface Props extends BadgeProps {
	readonly backend?: UnityScriptingBackend | null;
}

export function UnityBackendBadge(props: Props) {
	return (
		<Badge
			color={props.backend ? scriptingBackendColor[props.backend] : "dark"}
			{...props}
		>
			{props.backend ?? "-"}{" "}
		</Badge>
	);
}
