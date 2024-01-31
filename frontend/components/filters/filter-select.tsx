import { Button, Group, Stack, Tooltip } from "@mantine/core";
import { TableColumn } from "../table/table-head";
import { IconEye, IconEyeClosed, IconRestore } from "@tabler/icons-react";
import { useCallback, useMemo } from "react";
import { FilterButton } from "./filter-button";

const unknownOption = {
	value: null,
	label: "Unknown",
};

export type UnknownFilterOption = typeof unknownOption;

type Props<TKey extends string, TItem, TFilterOption extends string> = {
	readonly column: TableColumn<TKey, TItem, TFilterOption>;
	readonly visibleColumns: TKey[];
	readonly onChangeVisibleColumns: (visibleColumns: TKey[]) => void;
	readonly hiddenValues?: (TFilterOption | null)[];
	readonly onChange: (hiddenValues: (TFilterOption | null)[]) => void;
};

const defaultHiddenValues: unknown[] = [];

export function FilterSelect<
	TKey extends string,
	TItem,
	TFilterOption extends string,
>({
	column,
	onChange,
	onChangeVisibleColumns,
	visibleColumns,
	hiddenValues = defaultHiddenValues as TFilterOption[],
}: Props<TKey, TItem, TFilterOption>) {
	const optionsWithUnknown = useMemo(
		() => [unknownOption, ...(column.filterOptions ?? [])],
		[column.filterOptions],
	);

	const handleFilterClick = useCallback(
		(value: TFilterOption | null) => {
			const newValues =
				hiddenValues.indexOf(value) === -1
					? [...hiddenValues, value]
					: hiddenValues.filter((id) => id !== value);

			// If all possible values are hidden, it will always yield an empty result list,
			// so in that case we just reset this filter.
			onChange(newValues.length >= optionsWithUnknown.length ? [] : newValues);
		},
		[hiddenValues, onChange, optionsWithUnknown.length],
	);

	const handleReset = () => {
		onChange([]);
	};

	if (!column.hidable) return null;

	const isColumnVisible = visibleColumns.includes(column.id);

	return (
		<Stack gap="xs">
			<Tooltip
				openDelay={500}
				label="Toggle table column visibility"
			>
				<Button
					variant={isColumnVisible ? "filled" : "light"}
					leftSection={isColumnVisible ? <IconEye /> : <IconEyeClosed />}
					onClick={() =>
						onChangeVisibleColumns(
							isColumnVisible
								? visibleColumns.filter((col) => col !== column.id)
								: visibleColumns.concat(column.id),
						)
					}
				>
					<Group gap="xs">{column.label}</Group>
				</Button>
			</Tooltip>
			{column.filterOptions && (
				<>
					<Button.Group orientation="vertical">
						{optionsWithUnknown.map((filterOption) => (
							<FilterButton
								filterOption={filterOption}
								onClick={handleFilterClick}
								isHidden={hiddenValues.includes(filterOption.value)}
								isUnavailable={Boolean(
									filterOption.value &&
										column.unavailableValues?.includes(filterOption.value),
								)}
								key={filterOption.value}
							/>
						))}
					</Button.Group>
					<Button
						onClick={handleReset}
						leftSection={<IconRestore fontSize={10} />}
						disabled={(hiddenValues?.length || 0) === 0}
					>
						Reset
					</Button>
				</>
			)}
		</Stack>
	);
}
