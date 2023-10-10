import {
	Architecture,
	GameEngineBrand,
	UnityScriptingBackend,
} from "@api/bindings";
import { Badge, BadgeProps, DefaultMantineColor, Flex } from "@mantine/core";

interface Props<TValue extends string> extends BadgeProps {
	readonly value?: TValue | null;
	readonly label?: string;
}

type ColorRecord<TValue extends string> = Record<TValue, DefaultMantineColor>;

function CreateColorCodedBadge<TValue extends string>(
	fallbackText: string,
	colorMap: ColorRecord<TValue>,
) {
	return function ColorCodedBadge(props: Props<TValue>) {
		const color = props.value ? colorMap[props.value] : "dark";
		return (
			<Flex
				justify="center"
				w="100%"
			>
				<Badge
					color={color}
					style={
						props.label
							? {
									borderTopRightRadius: 0,
									borderBottomRightRadius: 0,
									paddingRight: 5,
									flex: 1,
							  }
							: undefined
					}
					{...props}
				>
					{props.value ?? fallbackText ?? "-"}
				</Badge>
				{props.label && (
					<Badge
						color="dark"
						variant="filled"
						style={{
							borderTopLeftRadius: 0,
							borderBottomLeftRadius: 0,
							paddingLeft: 5,
							opacity: 0.75,
							fontSize: "0.55rem",
							flex: 1,
							justifyContent: "start",
						}}
					>
						{props.label}
					</Badge>
				)}
			</Flex>
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
