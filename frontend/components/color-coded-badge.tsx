import {
	Architecture,
	GameEngineBrand,
	UnityScriptingBackend,
} from "@api/bindings";
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
		return (
			<Badge
				color={props.value ? colorMap[props.value] : "dark"}
				{...props}
			>
				{props.value ?? fallbackText ?? "-"}
			</Badge>
		);
	};
}

export const EngineBadge = CreateColorCodedBadge<GameEngineBrand>("Unknown", {
	Unity: "blue",
	Unreal: "red",
	Godot: "violet",
});

export const UnityBackendBadge = CreateColorCodedBadge<UnityScriptingBackend>(
	"-",
	{
		Il2Cpp: "red",
		Mono: "grape",
	},
);

export const ArchitectureBadge = CreateColorCodedBadge<Architecture>("X??", {
	X64: "blue",
	X86: "teal",
});

export const OperatingSystemBadge = CreateColorCodedBadge<string>("Unknown", {
	Linux: "yellow",
	Windows: "lime",
});
