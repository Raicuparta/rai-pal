import {
	Architecture,
	EngineBrand,
	GameTag,
	ProviderId,
	UnityScriptingBackend,
} from "@api/bindings";
import { Badge, BadgeProps, DefaultMantineColor, Flex } from "@mantine/core";
import styles from "./badges.module.css";
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
		const color = props.value ? colorMap[props.value] : "white";

		return (
			<Flex className={styles.wrapper}>
				<Badge
					color={color}
					className={
						props.value && props.label ? styles.labelledBadge : undefined
					}
					{...props}
				>
					{props.value ?? fallbackText}
				</Badge>
				{props.label && props.value && (
					<Badge
						color="dark"
						variant="filled"
						className={styles.label}
					>
						{props.label}
					</Badge>
				)}
			</Flex>
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
		Il2Cpp: "red",
		Mono: "grape",
	},
);

export const GameTagBadge = CreateColorCodedBadge<GameTag>("-", {
	VR: "green",
	Demo: "yellow",
});

export const ArchitectureBadge = CreateColorCodedBadge<Architecture>("-", {
	X64: "blue",
	X86: "teal",
});

export const ProviderBadge = CreateColorCodedBadge<ProviderId>("Unknown", {
	Manual: "gray",
	Steam: "blue",
	Epic: "red",
	Gog: "violet",
	Xbox: "green",
	Itch: "teal",
	Ubisoft: "grape",
	Ea: "yellow",
});
