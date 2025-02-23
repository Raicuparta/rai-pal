import { Button, Checkbox, Stack, Tooltip } from "@mantine/core";
import { GamesFilter } from "@api/bindings";
import { IconRestore } from "@tabler/icons-react";
import styles from "./filters.module.css";

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
	notes?: string;
	display?: string;
};

type FilterDetails<TKey extends keyof GamesFilter> = {
	title: string;

	// Text that shows for each filter type for the "empty value" option.
	// If not defined, the empty option is hidden from the filter menu.
	emptyOption?: string;

	valueDetails: Record<NonNullable<FilterValue<TKey>>, ValueDetails>;
};

const filterDetails: { [key in keyof GamesFilter]: FilterDetails<key> } = {
	architectures: {
		title: "Architecture",
		emptyOption: "Unknown",
		valueDetails: {
			X64: {
				display: "64-bit",
			},
			X86: {
				display: "32-bit",
			},
		},
	},
	engines: {
		title: "Engine",
		emptyOption: "Unknown",
		valueDetails: {
			Godot: {
				display: "Godot",
				notes: "Rai Pal doesn't fully support Godot yet.",
			},
			GameMaker: {
				display: "GameMaker",
				notes: "Rai Pal doesn't fully support GameMaker yet.",
			},
			Unity: {
				display: "Unity",
			},
			Unreal: {
				display: "Unreal",
			},
		},
	},
	unityScriptingBackends: {
		title: "Unity Backend",
		emptyOption: "Unknown",
		valueDetails: {
			Il2Cpp: {
				display: "IL2CPP",
			},
			Mono: {
				display: "Mono",
			},
		},
	},
	tags: {
		title: "Tags",
		emptyOption: "Untagged",
		valueDetails: {
			Demo: {
				display: "Demo",
			},
			VR: {
				display: "Native VR",
			},
		},
	},
	installed: {
		title: "Status",
		valueDetails: {
			Installed: {
				display: "Installed",
			},
			NotInstalled: {
				display: "Not Installed",
			},
		},
	},
	providers: {
		title: "Provider",
		valueDetails: {
			Ea: {
				display: "EA",
			},
			Epic: {
				display: "Epic",
			},
			Gog: {
				display: "GOG",
			},
			Itch: {
				display: "Itch.io",
			},
			Manual: {
				display: "Manual",
			},
			Steam: {
				display: "Steam",
			},
			Ubisoft: {
				display: "Ubisoft",
			},
			Xbox: {
				display: "Xbox",
				notes: "Xbox PC games only show on Rai Pal once they're installed.",
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
				<Button disabled>{filterDetails[id].title}</Button>
				{possibleValues.map((possibleValue) => {
					const valueDetails =
						possibleValue !== null
							? filterDetails[id].valueDetails[possibleValue]
							: undefined;
					return (
						<Tooltip
							key={possibleValue}
							label={valueDetails?.notes}
							disabled={!valueDetails?.notes}
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
								{valueDetails?.display ??
									filterDetails[id].emptyOption ??
									possibleValue}
								{valueDetails?.notes && " *"}
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
				Reset
			</Button>
		</Stack>
	);
}
