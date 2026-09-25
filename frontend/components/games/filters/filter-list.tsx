import { ActionIcon, Group, Stack, Text } from "@mantine/core";
import { useAtomValue } from "jotai";
import { IconX } from "@tabler/icons-react";
import styles from "./filter-list.module.css";
import {
	FilterChangeCallback,
	FilterColumn,
	hasDisabledNonLocked,
	keepOnlyLocked,
} from "./filter-column";
import { GamesFilter, GamesQuery } from "@api/bindings";
import { modsAtom } from "@hooks/use-data";
import { useDataQuery } from "@hooks/use-data-query";
import { filterDetails, FilterKey } from "./filter-details";
import { useLocalization } from "@hooks/use-localization";
import { defaultQuery } from "@hooks/default-settings";

export function FilterList() {
	const [dataQuery, setDataQuery] = useDataQuery();
	const mods = useAtomValue(modsAtom);
	const { t } = useLocalization("filterMenu");

	const filterKeys = Object.keys(filterDetails) as Array<FilterKey>;

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
		setDataQuery({ filter });
	};

	return (
		<Stack
			className={styles.root}
			gap="xs"
		>
			<Group
				justify="space-between"
				wrap="nowrap"
			>
				<Text
					fz="xs"
					fw={700}
					className={styles.filterTitle}
				>
					{t("button")}
				</Text>
				{hasActiveFilters && (
					<ActionIcon
						size="xs"
						variant="subtle"
						aria-label={t("resetButton")}
						onClick={handleResetAll}
					>
						<IconX size={13} />
					</ActionIcon>
				)}
			</Group>
			<Stack
				className={styles.scrollArea}
				gap="xs"
				align="start"
			>
				{filterKeys.map((filterKey) => {
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
						<FilterColumn
							key={filterKey}
							id={filterKey}
							possibleValues={possibleValues}
							filterGroup={dataQuery.filter[filterKey]}
							onChange={handleChange}
						/>
					);
				})}
			</Stack>
		</Stack>
	);
}
