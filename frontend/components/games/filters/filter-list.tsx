import { ActionIcon, Divider, Group, Stack } from "@mantine/core";
import { useAtomValue } from "jotai";
import { IconChevronUp, IconRestore } from "@tabler/icons-react";
import {
	FilterChangeCallback,
	FilterColumn,
	hasDisabledNonLocked,
	keepOnlyLocked,
} from "./filter-column";
import { GamesFilter, GamesQuery } from "@api/bindings";
import { modFamiliesAtom } from "@hooks/use-data";
import { useDataQuery } from "@hooks/use-data-query";
import { useAppSettingSingle } from "@hooks/use-app-setting-single";
import { filterDetails, FilterKey } from "./filter-details";
import { useLocalization } from "@hooks/use-localization";
import { defaultQuery } from "@hooks/default-settings";
import { ScrollAreaFill } from "@components/scroll-area-fill";

export function FilterList() {
	const [dataQuery, setDataQuery] = useDataQuery();
	const [expandedFilters = [], setExpandedFilters] =
		useAppSettingSingle("expandedFilters");
	const modFamilies = useAtomValue(modFamiliesAtom);
	const { t } = useLocalization("filterMenu");

	const hasModFamilies = Object.keys(modFamilies).length > 0;

	// When family definitions are unavailable, a saved family filter can't be
	// shown, so hide the category and don't apply it (but keep it saved).
	const filterKeys = (Object.keys(filterDetails) as Array<FilterKey>).filter(
		(filterKey) => filterKey !== "modFamilies" || hasModFamilies,
	);

	const hasActiveFilters = filterKeys.some((filterKey) =>
		hasDisabledNonLocked(dataQuery.filter[filterKey]),
	);

	const handleChange: FilterChangeCallback = (id, values) => {
		setDataQuery({
			filter: {
				...dataQuery.filter,
				[id]: values,
			},
		} as GamesQuery);
	};

	const handleResetAll = () => {
		const filter: GamesFilter = { ...defaultQuery.filter };
		for (const filterKey of filterKeys) {
			(filter as Record<string, unknown>)[filterKey] = keepOnlyLocked(
				dataQuery.filter[filterKey],
			);
		}
		if (!hasModFamilies) {
			filter.modFamilies = dataQuery.filter.modFamilies;
		}
		setDataQuery({ filter });
	};

	return (
		<Stack
			flex={1}
			mih={100}
		>
			<Group
				wrap="nowrap"
				pr="xs"
			>
				<ActionIcon
					bg="transparent"
					size="sm"
					variant="subtle"
					disabled={!hasActiveFilters}
					aria-label={t("resetButton")}
					onClick={handleResetAll}
				>
					<IconRestore fontSize={16} />
				</ActionIcon>
				<Divider
					label={t("filters")}
					w="100%"
				/>
				<ActionIcon
					bg="transparent"
					size="sm"
					mr={-3}
					color="gray"
					variant="subtle"
					disabled={expandedFilters.length === 0}
					aria-label={t("collapseAll")}
					onClick={() => setExpandedFilters([])}
				>
					<IconChevronUp fontSize={16} />
				</ActionIcon>
			</Group>
			<ScrollAreaFill
				flex={1}
				mih={0}
			>
				<Stack gap={0}>
					{filterKeys.map((filterKey) => {
						const possibleValues =
							filterKey === "modFamilies"
								? (Object.keys(modFamilies) as string[])
								: (Object.keys(
										filterDetails[filterKey].valueDetails,
									) as string[]);

						const valueDetails =
							filterKey === "modFamilies"
								? Object.fromEntries(
										Object.entries(modFamilies).map(([id, family]) => [
											id,
											{ staticDisplayText: family.title },
										]),
									)
								: undefined;

						return (
							<FilterColumn
								key={filterKey}
								id={filterKey}
								possibleValues={possibleValues}
								valueDetails={valueDetails}
								filterGroup={dataQuery.filter[filterKey]}
								expanded={expandedFilters.includes(filterKey)}
								onExpandedChange={(expanded) =>
									setExpandedFilters((prev = []) =>
										expanded
											? [...prev, filterKey]
											: prev.filter((key) => key !== filterKey),
									)
								}
								onChange={handleChange}
							/>
						);
					})}
				</Stack>
			</ScrollAreaFill>
		</Stack>
	);
}
