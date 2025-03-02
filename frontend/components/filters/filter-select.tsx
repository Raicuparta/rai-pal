import { Button, Checkbox, Stack, Tooltip } from "@mantine/core";
import { GamesFilter } from "@api/bindings";
import { IconRestore } from "@tabler/icons-react";
import styles from "./filters.module.css";
import { TranslationKey, useGetTranslated } from "@hooks/use-translations";

export type FilterKey = keyof GamesFilter;
export type FilterValue<TFilterKey extends FilterKey> =
	GamesFilter[TFilterKey][number];

export type FilterChangeCallback = (
	id: keyof GamesFilter,
	value: Array<FilterValue<typeof id>>,
) => void;

type Props<TFilterKey extends keyof GamesFilter> = {
	readonly id: TFilterKey;
	readonly possibleValues: Array<FilterValue<TFilterKey>>;
	readonly currentValues: Array<FilterValue<TFilterKey>>;
	readonly onChange: FilterChangeCallback;
};

type ValueDetails = {
	noteTranslationKey?: TranslationKey<"filterValueNote">;
	translationKey?: TranslationKey<"filterValue">;
	staticDisplayText?: string;
};

type FilterDetails<TKey extends keyof GamesFilter> = {
	translationKey: TranslationKey<"filterProperty">;

	// Text that shows for each filter type for the "empty value" option.
	// If not defined, the empty option is hidden from the filter menu.
	emptyTranslationKey?: TranslationKey<"filterValue">;

	valueDetails: Record<NonNullable<FilterValue<TKey>>, ValueDetails>;
};

const filterDetails: { [key in keyof GamesFilter]: FilterDetails<key> } = {
	architectures: {
		translationKey: "architecture",
		emptyTranslationKey: "unknown",
		valueDetails: {
			X64: {
				translationKey: "arch64",
			},
			X86: {
				translationKey: "arch32",
			},
		},
	},
	engines: {
		translationKey: "engine",
		emptyTranslationKey: "unknown",
		valueDetails: {
			Godot: {
				staticDisplayText: "Godot",
				noteTranslationKey: "engineGodotNotFullySupported",
			},
			GameMaker: {
				staticDisplayText: "GameMaker",
				noteTranslationKey: "engineGameMakerNotFullySupported",
			},
			Unity: {
				staticDisplayText: "Unity",
			},
			Unreal: {
				staticDisplayText: "Unreal",
			},
		},
	},
	unityScriptingBackends: {
		translationKey: "unityScriptingBackend",
		emptyTranslationKey: "unknown",
		valueDetails: {
			Il2Cpp: {
				staticDisplayText: "IL2CPP",
			},
			Mono: {
				staticDisplayText: "Mono",
			},
		},
	},
	tags: {
		translationKey: "tags",
		emptyTranslationKey: "tagUntagged",
		valueDetails: {
			Demo: {
				translationKey: "tagDemo",
			},
			VR: {
				translationKey: "tagVr",
			},
		},
	},
	installed: {
		translationKey: "status",
		valueDetails: {
			Installed: {
				translationKey: "statusInstalled",
			},
			NotInstalled: {
				translationKey: "statusNotInstalled",
			},
		},
	},
	providers: {
		translationKey: "provider",
		valueDetails: {
			Ea: {
				staticDisplayText: "EA",
			},
			Epic: {
				staticDisplayText: "Epic",
			},
			Gog: {
				staticDisplayText: "GOG",
			},
			Itch: {
				staticDisplayText: "itch.io",
			},
			Manual: {
				translationKey: "providerManual",
			},
			Steam: {
				staticDisplayText: "Steam",
			},
			Ubisoft: {
				staticDisplayText: "Ubisoft",
			},
			Xbox: {
				staticDisplayText: "Xbox",
				noteTranslationKey: "providerXboxOnlyInstalled",
			},
		},
	},
};

export function FilterSelect<TFilterKey extends FilterKey>({
	id,
	possibleValues,
	currentValues,
	onChange,
}: Props<TFilterKey>) {
	const tMenu = useGetTranslated("filterMenu");
	const tProperty = useGetTranslated("filterProperty");
	const tValue = useGetTranslated("filterValue");
	const tValueNote = useGetTranslated("filterValueNote");

	function handleFilterClick(id: TFilterKey, value: string | null) {
		const newValues = [...currentValues];
		const index = newValues.indexOf(value as FilterValue<TFilterKey>);
		if (index === -1) {
			newValues.push(value as FilterValue<TFilterKey>);
		} else {
			newValues.splice(index, 1);
		}

		onChange(id, newValues);
	}

	function handleResetClick() {
		onChange(id, possibleValues);
	}

	return (
		<Stack gap="xs">
			<Button.Group orientation="vertical">
				<Button disabled>{tProperty(filterDetails[id].translationKey)}</Button>
				{possibleValues.map((possibleValue) => {
					const valueDetails =
						possibleValue !== null
							? filterDetails[id].valueDetails[possibleValue]
							: undefined;
					return (
						<Tooltip
							key={possibleValue}
							label={tValueNote(valueDetails?.noteTranslationKey)}
							disabled={!valueDetails?.noteTranslationKey}
						>
							<Button
								fullWidth
								justify="start"
								leftSection={
									<Checkbox
										tabIndex={-1}
										readOnly
										className={styles.checkbox}
										checked={currentValues.includes(possibleValue)}
									/>
								}
								onClick={() => handleFilterClick(id, possibleValue)}
							>
								{possibleValue === null
									? tValue(filterDetails[id].emptyTranslationKey)
									: (valueDetails?.staticDisplayText ??
										tValue(valueDetails?.translationKey) ??
										possibleValue)}
								{valueDetails?.noteTranslationKey && " *"}
							</Button>
						</Tooltip>
					);
				})}
			</Button.Group>
			<Button
				onClick={handleResetClick}
				leftSection={<IconRestore fontSize={10} />}
				disabled={(currentValues?.length || 0) === possibleValues.length}
			>
				{tMenu("resetButton")}
			</Button>
		</Stack>
	);
}
