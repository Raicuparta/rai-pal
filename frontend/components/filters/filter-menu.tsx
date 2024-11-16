import { Button, Group, Indicator, Popover } from "@mantine/core";
import { IconFilter, IconX } from "@tabler/icons-react";
import styles from "./filters.module.css";
import { useCallback, useEffect, useState } from "react";
import { type Result, type Error } from "@api/bindings";
import { FilterSelect } from "./filter-select";

type Filter = Record<string, Record<string, boolean>>;

type Props<TFilter extends Filter> = {
	readonly setterCommand: (filter: TFilter) => Promise<Result<null, Error>>;
	readonly getterCommand: () => Promise<Result<TFilter, Error>>;
};

export function FilterMenu<TFilter extends Filter>({
	setterCommand,
	getterCommand,
}: Props<TFilter>) {
	const [currentFilter, setCurrentFilter] = useState<TFilter>();

	const updateFilters = useCallback(() => {
		getterCommand().then((result) => {
			console.log("aa?", result);
			if (result.status === "ok") {
				setCurrentFilter(result.data);
			}
		});
	}, [getterCommand]);

	const handleFilterOptionClick = useCallback(
		(id: string, value: string) => {
			setterCommand({
				...currentFilter,
				[id]: {
					...currentFilter?.[id],
					[value]: !currentFilter?.[id]?.[value],
				},
			} as TFilter);

			updateFilters();
		},
		[currentFilter, setterCommand, updateFilters],
	);

	useEffect(() => {
		updateFilters();
	}, [updateFilters]);

	const active = false;

	return (
		<Indicator
			disabled={!active}
			offset={8}
		>
			<Button.Group>
				{active && (
					<Button
						// onClick={() => setFilter(undefined)}
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
										onClick={handleFilterOptionClick}
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
