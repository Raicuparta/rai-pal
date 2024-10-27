import {
	Architecture,
	EngineBrand,
	GameTag,
	ProviderId,
	UnityScriptingBackend,
} from "@api/bindings";
import { Badge, BadgeProps, DefaultMantineColor, Flex } from "@mantine/core";
import styles from "./badges.module.css";
import {
	engineFilterOptions,
	providerFilterOptions,
} from "@util/common-filter-options";
import { FilterOption } from "@components/table/table-head";

interface Props<TValue extends string> extends BadgeProps {
	readonly value?: TValue | null;
	readonly label?: string;
}

type ColorRecord<TValue extends string> = Record<TValue, DefaultMantineColor>;

function CreateColorCodedBadge<TValue extends string>(
	fallbackText: string,
	colorMap: ColorRecord<TValue>,
	filterOptions?: FilterOption<TValue>[],
) {
	return function ColorCodedBadge(props: Props<TValue>) {
		const color = props.value ? colorMap[props.value] : "white";
		const label =
			props.value && filterOptions
				? filterOptions.find((option) => option.value === props.value)?.label
				: undefined;

		return (
			<Flex className={styles.wrapper}>
				<Badge
					color={color}
					className={
						props.value && props.label ? styles.labelledBadge : undefined
					}
					{...props}
				>
					{label ?? props.value ?? fallbackText}
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

export const EngineBadge = CreateColorCodedBadge<EngineBrand>(
	"-",
	{
		Unity: "blue",
		Unreal: "red",
		Godot: "violet",
		GameMaker: "teal",
	},
	engineFilterOptions,
);

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

export const ProviderBadge = CreateColorCodedBadge<ProviderId>(
	"Unknown",
	{
		Manual: "gray",
		Steam: "blue",
		Epic: "red",
		Gog: "violet",
		Xbox: "green",
		Itch: "teal",
		// Ubisoft: "grape",
	},
	providerFilterOptions,
);
