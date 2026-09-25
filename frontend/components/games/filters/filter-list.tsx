import { Stack } from "@mantine/core";
import { useAtomValue } from "jotai";
import styles from "./filter-list.module.css";
import { FilterChangeCallback, FilterColumn } from "./filter-column";
import { GamesQuery } from "@api/bindings";
import { modsAtom } from "@hooks/use-data";
import { useDataQuery } from "@hooks/use-data-query";
import { filterDetails, FilterKey } from "./filter-details";

export function FilterList() {
	const [dataQuery, setDataQuery] = useDataQuery();
	const mods = useAtomValue(modsAtom);

	const handleChange: FilterChangeCallback = (id, values) => {
		setDataQuery({
			filter: {
				...dataQuery.filter,
				[id]: values,
			},
		} as GamesQuery);
	};

	return (
		<Stack
			className={styles.root}
			gap="xs"
			align="start"
		>
			{(Object.keys(filterDetails) as Array<FilterKey>).map((filterKey) => {
				const possibleValues =
					filterKey === "modFamilies"
						? ([
								...new Set(
									Object.values(mods)
										.map((m) => m.family)
										.filter((f): f is string => f !== null),
								),
							] as string[])
						: (Object.keys(filterDetails[filterKey].valueDetails) as string[]);

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
	);
}
