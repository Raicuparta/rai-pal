import { Button, Group, Indicator, Popover } from "@mantine/core";
import { IconFilter, IconX } from "@tabler/icons-react";
import styles from "./filters.module.css";
import { useCallback } from "react";
import { FilterChangeCallback, FilterKey, FilterSelect } from "./filter-select";
import { SearchInput } from "@components/search-input";
import { GamesFilter, GamesQuery } from "@api/bindings";
import { defaultQuery, useDataQuery } from "@hooks/use-data-query";
import { useLocalization } from "@hooks/use-localization";

export function FilterMenu() {
	const [dataQuery, setDataQuery] = useDataQuery();
	const t = useLocalization("filterMenu");

	const handleToggleClick = useCallback<FilterChangeCallback>(
		(id, values) => {
			setDataQuery({
				filter: {
					...dataQuery?.filter,
					[id]: values.length > 0 ? values : defaultQuery.filter[id],
				},
			} as GamesQuery);
		},
		[dataQuery, setDataQuery],
	);

	const handleReset = useCallback(() => {
		setDataQuery(null);
	}, [setDataQuery]);

	const handleSearchChange = useCallback(
		(search: string) => {
			setDataQuery({
				search,
			});
		},
		[setDataQuery],
	);

	// active if has search or any filter has length smaller than default
	const active =
		dataQuery.search.length > 0 ||
		Object.keys(dataQuery.filter).some(
			(filterId) =>
				dataQuery.filter[filterId as keyof GamesFilter].length <
				defaultQuery.filter[filterId as keyof GamesFilter].length,
		);

	return (
		<>
			<SearchInput
				onChange={handleSearchChange}
				value={dataQuery.search}
			/>
			<Indicator
				disabled={!active}
				offset={8}
			>
				<Button.Group>
					{active && (
						<Button
							onClick={handleReset}
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
							bg="dark"
							p={0}
							className={styles.dropdown}
						>
							<Group
								className={styles.dropdownContent}
								p="xs"
								align="start"
								wrap="nowrap"
							>
								{(Object.keys(defaultQuery.filter) as Array<FilterKey>).map(
									(filterId) => (
										<FilterSelect
											key={filterId}
											id={filterId}
											possibleValues={defaultQuery.filter[filterId]}
											currentValues={dataQuery.filter[filterId]}
											onChange={handleToggleClick}
										/>
									),
								)}
							</Group>
						</Popover.Dropdown>
					</Popover>
				</Button.Group>
			</Indicator>
		</>
	);
}
