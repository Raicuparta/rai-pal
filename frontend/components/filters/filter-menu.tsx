import { Button, Group, Indicator, Popover } from "@mantine/core";
import { IconFilter, IconX } from "@tabler/icons-react";
import styles from "./filters.module.css";
import { useCallback, useEffect, useState } from "react";
import { type Result, type Error } from "@api/bindings";
import { FilterSelect } from "./filter-select";
import { SearchInput } from "@components/search-input";

type Filter = {
	toggles: Record<string, Record<string, boolean>>;
	search: string;
};

type Props<TFilter extends Filter> = {
	readonly setterCommand: (
		filter: TFilter | null,
	) => Promise<Result<null, Error>>;
	readonly getterCommand: () => Promise<Result<TFilter, Error>>;
};

export function FilterMenu<TFilter extends Filter>({
	setterCommand,
	getterCommand,
}: Props<TFilter>) {
	const [currentFilter, setCurrentFilter] = useState<TFilter>();
	const [currentSearch, setCurrentSearch] = useState<string>("");

	const updateFilters = useCallback(() => {
		getterCommand().then((result) => {
			if (result.status === "ok") {
				setCurrentFilter(result.data);
			}
		});
	}, [getterCommand]);

	const handleToggleClick = useCallback(
		(id: string, value: string) => {
			setterCommand({
				...currentFilter,
				toggles: {
					...currentFilter?.toggles,
					[id]: {
						...currentFilter?.toggles?.[id],
						[value]: !currentFilter?.toggles?.[id]?.[value],
					},
				},
			} as TFilter);

			updateFilters();
		},
		[currentFilter, setterCommand, updateFilters],
	);

	const handleReset = useCallback(() => {
		setCurrentSearch("");
		setterCommand(null).then(updateFilters);
	}, [setterCommand, updateFilters]);

	const handleSearchChange = useCallback(
		(search: string) => {
			setCurrentSearch(search);

			setterCommand({
				...currentFilter,
				search,
			} as TFilter).then(updateFilters);
		},
		[currentFilter, setterCommand, updateFilters],
	);

	useEffect(() => {
		updateFilters();
	}, [updateFilters]);

	// active if has search or any toggle is set to false:
	const active =
		Boolean(currentSearch) ||
		Object.values(currentFilter?.toggles ?? {}).some((toggleGroup) =>
			Object.values(toggleGroup).some((value) => !value),
		);

	return (
		<>
			<SearchInput
				onChange={handleSearchChange}
				value={currentSearch}
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
								{Object.entries(currentFilter?.toggles ?? {}).map(
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
