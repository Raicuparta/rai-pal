import { Button, Group, Indicator, Popover } from "@mantine/core";
import { IconFilter, IconX } from "@tabler/icons-react";
import styles from "./filters.module.css";
import { FilterChangeCallback, FilterKey, FilterSelect, FilterValue } from "./filter-select";
import { SearchInput } from "@components/search-input";
import { GamesFilter, GamesQuery } from "@api/bindings";
import { useDataQuery } from "@hooks/use-data-query";
import { useLocalization } from "@hooks/use-localization";
import { LocalizationKey } from "@localizations/localizations";

type ValueDetails = {
	noteLocalizationKey?: LocalizationKey<"filterValueNote">;
	localizationKey?: LocalizationKey<"filterValue">;
	staticDisplayText?: string;
};

type FilterDetails<TKey extends FilterKey> = {
	localizationKey: LocalizationKey<"filterProperty">;

	// Text that shows for each filter type for the "empty value" option.
	// If not defined, the empty option is hidden from the filter menu.
	emptyLocalizationKey?: LocalizationKey<"filterValue">;

	valueDetails: Record<NonNullable<FilterValue<TKey>>, ValueDetails>;
};

export const filterDetails = Object.freeze<{ [key in FilterKey]: FilterDetails<key> }>({
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
});

export function FilterMenu() {
	const [dataQuery, setDataQuery] = useDataQuery();
	const t = useLocalization("filterMenu");

	const handleToggleClick: FilterChangeCallback = (id, values) => {
		setDataQuery({
			filter: {
				...dataQuery?.filter,
				[id]: values,
			},
		} as GamesQuery);
	};

	// active if has search or any filter has length smaller than default
	const active =
		Object.keys(dataQuery.filter).some(
			(filterId) =>
				dataQuery.filter[filterId as keyof GamesFilter].length > 0
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
							onClick={() => setDataQuery(null)}
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
								{(Object.keys(filterDetails) as Array<FilterKey>).map((filterKey) => (
									<FilterSelect
										key={filterKey}
										id={filterKey}
										possibleValues={Object.keys(filterDetails[filterKey].valueDetails) as Array<NonNullable<FilterValue<FilterKey>>>}
										currentValues={dataQuery.filter[filterKey]}
										onChange={handleToggleClick}
									/>
								))}
							</Group>
						</Popover.Dropdown>
					</Popover>
				</Button.Group>
			</Indicator>
		</>
	);
}
