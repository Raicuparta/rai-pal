import { Button, Checkbox, Stack, Tooltip } from "@mantine/core";
import { GamesFilter } from "@api/bindings";
import { IconRestore } from "@tabler/icons-react";
import styles from "./filters.module.css";
import { useLocalization } from "@hooks/use-localization";
import { LocalizationKey } from "@localizations/localizations";

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
	noteLocalizationKey?: LocalizationKey<"filterValueNote">;
	localizationKey?: LocalizationKey<"filterValue">;
	staticDisplayText?: string;
};

type FilterDetails<TKey extends keyof GamesFilter> = {
	localizationKey: LocalizationKey<"filterProperty">;

	// Text that shows for each filter type for the "empty value" option.
	// If not defined, the empty option is hidden from the filter menu.
	emptyLocalizationKey?: LocalizationKey<"filterValue">;

	valueDetails: Record<NonNullable<FilterValue<TKey>>, ValueDetails>;
};

const filterDetails: { [key in keyof GamesFilter]: FilterDetails<key> } = {
	architectures: {
		localizationKey: "architecture",
		emptyLocalizationKey: "unknown",
		valueDetails: {
			X64: {
				localizationKey: "arch64",
			},
			X86: {
				localizationKey: "arch32",
			},
		},
	},
	engines: {
		localizationKey: "engine",
		emptyLocalizationKey: "unknown",
		valueDetails: {
			Godot: {
				staticDisplayText: "Godot",
				noteLocalizationKey: "engineGodotNotFullySupported",
			},
			GameMaker: {
				staticDisplayText: "GameMaker",
				noteLocalizationKey: "engineGameMakerNotFullySupported",
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
		localizationKey: "unityScriptingBackend",
		emptyLocalizationKey: "unknown",
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
		localizationKey: "tags",
		emptyLocalizationKey: "tagUntagged",
		valueDetails: {
			Demo: {
				localizationKey: "tagDemo",
			},
			VR: {
				localizationKey: "tagVr",
			},
		},
	},
	installed: {
		localizationKey: "status",
		valueDetails: {
			Installed: {
				localizationKey: "statusInstalled",
			},
			NotInstalled: {
				localizationKey: "statusNotInstalled",
			},
		},
	},
	providers: {
		localizationKey: "provider",
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
				localizationKey: "providerManual",
			},
			Steam: {
				staticDisplayText: "Steam",
			},
			Ubisoft: {
				staticDisplayText: "Ubisoft",
			},
			Xbox: {
				staticDisplayText: "Xbox",
				noteLocalizationKey: "providerXboxOnlyInstalled",
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
	const tMenu = useLocalization("filterMenu");
	const tProperty = useLocalization("filterProperty");
	const tValue = useLocalization("filterValue");
	const tValueNote = useLocalization("filterValueNote");

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
				<Button disabled>{tProperty(filterDetails[id].localizationKey)}</Button>
				{possibleValues.map((possibleValue) => {
					const valueDetails =
						possibleValue !== null
							? filterDetails[id].valueDetails[possibleValue]
							: undefined;
					return (
						<Tooltip
							key={possibleValue}
							label={tValueNote(valueDetails?.noteLocalizationKey)}
							disabled={!valueDetails?.noteLocalizationKey}
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
									? tValue(filterDetails[id].emptyLocalizationKey)
									: (valueDetails?.staticDisplayText ??
										tValue(valueDetails?.localizationKey) ??
										possibleValue)}
								{valueDetails?.noteLocalizationKey && " *"}
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
