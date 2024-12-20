import { Button, Group, Indicator, Popover } from "@mantine/core";
import { IconFilter, IconX } from "@tabler/icons-react";
import styles from "./filters.module.css";
import { useCallback } from "react";
import { FilterSelect } from "./filter-select";
import { SearchInput } from "@components/search-input";
import { GamesFilter, GamesQuery } from "@api/bindings";
import { useDataQuery } from "@hooks/use-data-query";

export function FilterMenu() {
	const [dataQuery, setDataQuery] = useDataQuery();

	const handleToggleClick = useCallback(
		(id: keyof GamesFilter, value: string) => {
			setDataQuery({
				filter: {
					...dataQuery?.filter,
					[id]: {
						...dataQuery?.filter?.[id],
						[value]: !dataQuery?.filter?.[id]?.[value],
					},
				},
			} as GamesQuery);
		},
		[dataQuery?.filter, setDataQuery],
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

	// active if has search or any toggle is set to false:
	const active =
		Boolean(dataQuery?.search) ||
		Object.values(dataQuery?.filter ?? {}).some((toggleGroup) =>
			Object.values(toggleGroup).some((value) => !value),
		);

	return (
		<>
			<SearchInput
				onChange={handleSearchChange}
				value={dataQuery?.search ?? ""}
				count={999}
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
							<Button leftSection={<IconFilter />}>Filter</Button>
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
								{Object.entries(dataQuery?.filter ?? {}).map(
									([filterId, filterOptions]) => (
										<FilterSelect
											key={filterId}
											id={filterId}
											filterOptions={filterOptions}
											onClick={handleToggleClick}
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
