import { Button, InputLabel, Stack } from "@mantine/core";
import { GamesFilter } from "@api/bindings";
import { IconRestore } from "@tabler/icons-react";
import { useLocalization } from "@hooks/use-localization";
import { LocalizationKey } from "@localizations/localizations";
import { CheckboxButton } from "@components/checkbox-button";

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
				noteLocalizationKey: "providerUbisoftOnlySubscription",
			},
			Xbox: {
				staticDisplayText: "Xbox",
				noteLocalizationKey: "providerXboxOnlyInstalledAndSubscription",
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
		console.log("wtf");
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
		<Stack>
			<Stack
				gap={0}
				align="center"
			>
				<InputLabel>{tProperty(filterDetails[id].localizationKey)}</InputLabel>
				<Button.Group orientation="vertical">
					{possibleValues.map((possibleValue) => {
						const valueDetails =
							possibleValue !== null
								? filterDetails[id].valueDetails[possibleValue]
								: undefined;
						return (
							<CheckboxButton
								key={possibleValue}
								tooltip={tValueNote(valueDetails?.noteLocalizationKey)}
								checked={currentValues.includes(possibleValue)}
								onChange={() => handleFilterClick(id, possibleValue)}
							>
								{possibleValue === null
									? tValue(filterDetails[id].emptyLocalizationKey)
									: (valueDetails?.staticDisplayText ??
										tValue(valueDetails?.localizationKey) ??
										possibleValue)}
							</CheckboxButton>
						);
					})}
				</Button.Group>
			</Stack>
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
