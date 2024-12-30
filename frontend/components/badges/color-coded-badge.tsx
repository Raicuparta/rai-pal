import { EngineBrand, UnityScriptingBackend } from "@api/bindings";
import { Badge, BadgeProps, DefaultMantineColor } from "@mantine/core";
interface Props<TValue extends string> extends BadgeProps {
	readonly value?: TValue | null;
}

type ColorRecord<TValue extends string> = Record<TValue, DefaultMantineColor>;

function CreateColorCodedBadge<TValue extends string>(
	fallbackText: string,
	colorMap: ColorRecord<TValue>,
) {
	return function ColorCodedBadge(props: Props<TValue>) {
		const color = props.value ? colorMap[props.value] : "white";

		return (
			<Badge
				color={color}
				{...props}
			>
				{props.value ?? fallbackText}
			</Badge>
		);
	};
}

export const EngineBadge = CreateColorCodedBadge<EngineBrand>("-", {
	Unity: "blue",
	Unreal: "red",
	Godot: "violet",
	GameMaker: "teal",
});

export const UnityBackendBadge = CreateColorCodedBadge<UnityScriptingBackend>(
	"-",
	{
		Il2Cpp: "grape",
		Mono: "cyan",
	},
);
