// TODO: test this after fresh start (delete app data).

import { Button, Group, Indicator, Popover } from "@mantine/core";
import { IconFilter, IconX } from "@tabler/icons-react";
import { useAtomValue } from "jotai";
import styles from "./filters.module.css";
import {
	FilterChangeCallback,
	FilterKey,
	FilterSelect,
	keepOnlyLocked,
} from "./filter-select";
import { SearchInput } from "@components/search-input";
import { FilterGroup, GamesFilter, GamesQuery } from "@api/bindings";
import { modsAtom } from "@hooks/use-data";
import { defaultQuery, useDataQuery } from "@hooks/use-data-query";
import { useLocalization } from "@hooks/use-localization";
import { LocalizationKey } from "@localizations/localizations";

type ValueDetails = {
	noteLocalizationKey?: LocalizationKey<"filterValueNote">;
	localizationKey?: LocalizationKey<"filterValue">;
	staticDisplayText?: string;
};

type FilterDetails = {
	localizationKey: LocalizationKey<"filterProperty">;

	// Text that shows for each filter type for the "empty value" option.
	// If not defined, the empty option is hidden from the filter menu.
	emptyLocalizationKey?: LocalizationKey<"filterValue">;

	valueDetails: Record<string, ValueDetails>;
};

export const filterDetails = Object.freeze<{
	[key in FilterKey]: FilterDetails;
}>({
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
	unityBackends: {
		localizationKey: "unityBackend",
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
	os: {
		localizationKey: "os",
		emptyLocalizationKey: "unknown",
		valueDetails: {
			Windows: {
				staticDisplayText: "Windows",
			},
			Linux: {
				staticDisplayText: "Linux",
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
			Xbox: {
				staticDisplayText: "Xbox",
				noteLocalizationKey: "providerXboxOnlyInstalled",
			},
		},
	},
	modFamilies: {
		localizationKey: "mod",
		valueDetails: {},
	},
});

function hasDisabledNonLocked(group: FilterGroup<string>): boolean {
	return (
		Object.values(group.known).some((item) => !item.enabled && !item.locked) ||
		(group.unknown !== null && !group.unknown.enabled && !group.unknown.locked)
	);
}

export function FilterMenu() {
	const [dataQuery, setDataQuery] = useDataQuery();
	const mods = useAtomValue(modsAtom);
	const { t } = useLocalization("filterMenu");

	const handleToggleClick: FilterChangeCallback = (id, values) => {
		setDataQuery({
			filter: {
				...dataQuery?.filter,
				[id]: values,
			},
		} as GamesQuery);
	};

	const active = (Object.keys(filterDetails) as FilterKey[]).some((filterId) =>
		hasDisabledNonLocked(dataQuery.filter[filterId] as FilterGroup<string>),
	);

	return (
		<>
			<SearchInput
				onChange={(search) => {
					setDataQuery({
						search,
					});
				}}
				value={dataQuery.search}
			/>
			<Indicator
				disabled={!active}
				offset={8}
			>
				<Button.Group>
					{active && (
						<Button
							onClick={() => {
								const newFilter: GamesFilter = {
									...defaultQuery.filter,
								};

								for (const key of Object.keys(filterDetails) as FilterKey[]) {
									(newFilter as Record<string, unknown>)[key] = keepOnlyLocked(
										dataQuery.filter[key] as FilterGroup<string>,
									);
								}

								setDataQuery({
									filter: newFilter,
								});
							}}
							px={5}
						>
							<IconX />
						</Button>
					)}
					<Popover
						trapFocus
						position="bottom-end"
					>
						<Popover.Target>
							<Button leftSection={<IconFilter />}>{t("button")}</Button>
						</Popover.Target>
						<Popover.Dropdown
							p={0}
							className={styles.dropdown}
						>
							<Group
								className={styles.dropdownContent}
								p="xs"
								align="start"
								wrap="nowrap"
							>
								{(Object.keys(filterDetails) as Array<FilterKey>).map(
									(filterKey) => {
										const possibleValues =
											filterKey === "modFamilies"
												? ([
														...new Set(
															Object.values(mods)
																.map((m) => m.family)
																.filter((f): f is string => f !== null),
														),
													] as string[])
												: (Object.keys(
														filterDetails[filterKey].valueDetails,
													) as string[]);

										return (
											<FilterSelect
												key={filterKey}
												id={filterKey}
												possibleValues={possibleValues}
												filterGroup={
													dataQuery.filter[filterKey] as FilterGroup<string>
												}
												onChange={handleToggleClick}
											/>
										);
									},
								)}
							</Group>
						</Popover.Dropdown>
					</Popover>
				</Button.Group>
			</Indicator>
		</>
	);
}
