import { Button, Group, Indicator, Popover } from "@mantine/core";
import { IconFilter, IconX } from "@tabler/icons-react";
import styles from "./filters.module.css";
import { useCallback, useEffect, useState } from "react";
import { commands, InstalledGamesFilter } from "@api/bindings";
import { FilterSelect } from "./filter-select";

type Props = {
	readonly active: boolean;
	readonly setFilter: (filter: undefined) => void;
};

export function FilterMenu(props: Props) {
	const [currentFilter, setCurrentFilter] = useState<InstalledGamesFilter>();

	const updateFilters = useCallback(() => {
		commands.getInstalledGamesFilter().then((result) => {
			console.log("result here ", result);
			if (result.status === "ok") {
				setCurrentFilter(result.data);
			}
		});
	}, []);

	useEffect(() => {
		updateFilters();
	}, [updateFilters]);

	return (
		<Indicator
			disabled={!props.active}
			offset={8}
		>
			<Button.Group>
				{props.active && (
					<Button
						onClick={() => props.setFilter(undefined)}
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
							{Object.entries(currentFilter ?? {}).map(
								([filterId, filterOptions]) => (
									<FilterSelect
										key={filterId}
										id={filterId}
										filterOptions={filterOptions}
										// onChangeVisibleColumns={setVisibleColumnIds}
										onClick={(id, value) => {
											commands.setInstalledGamesFilter({
												...currentFilter,
												[id]: {
													...currentFilter?.[id],
													[value]: !currentFilter?.[id]?.[value],
												},
											});

											updateFilters();
										}}
									/>
								),
							)}
						</Group>
					</Popover.Dropdown>
				</Popover>
			</Button.Group>
		</Indicator>
	);
}
