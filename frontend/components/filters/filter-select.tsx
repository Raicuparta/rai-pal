import { Button, Group, Stack, Tooltip } from "@mantine/core";
import { TableColumn } from "../table/table-head";
import {
	IconEye,
	IconEyeClosed,
	IconRestore,
	IconSquare,
	IconSquareCheck,
} from "@tabler/icons-react";

type Props<TKey extends string, TItem, TFilterOption extends string> = {
	readonly column: TableColumn<TKey, TItem, TFilterOption>;
	readonly visibleColumns: TKey[];
	readonly onChangeVisibleColumns: (visibleColumns: TKey[]) => void;
	readonly hiddenValues?: (TFilterOption | null)[];
	readonly onChange: (hiddenValues: (TFilterOption | null)[]) => void;
};

const defaultValue: unknown[] = [];

export function FilterSelect<
	TKey extends string,
	TItem,
	TFilterOption extends string,
>(props: Props<TKey, TItem, TFilterOption>) {
	const selectedValues =
		props.hiddenValues ?? (defaultValue as TFilterOption[]);

	const isHidden = (value: TFilterOption | null) => {
		return selectedValues.includes(value);
	};

	const handleChange = (newValue: TFilterOption | null) => {
		props.onChange(
			selectedValues.indexOf(newValue) === -1
				? [...selectedValues, newValue]
				: selectedValues.filter((id) => id !== newValue),
		);
	};

	const handleReset = () => {
		props.onChange([]);
	};

	if (!props.column.filterOptions) return null;

	const isColumnVisible = props.visibleColumns.includes(props.column.id);

	return (
		<Stack gap="xs">
			<Tooltip
				openDelay={500}
				label="Toggle table column visibility"
			>
				<Button
					size="compact-sm"
					variant={isColumnVisible ? "filled" : "light"}
					leftSection={isColumnVisible ? <IconEye /> : <IconEyeClosed />}
					onClick={() =>
						props.onChangeVisibleColumns(
							isColumnVisible
								? props.visibleColumns.filter((col) => col !== props.column.id)
								: props.visibleColumns.concat(props.column.id),
						)
					}
				>
					<Group gap="xs">{props.column.label}</Group>
				</Button>
			</Tooltip>
			<Button.Group orientation="vertical">
				<Button
					variant={isHidden(null) ? "light" : "filled"}
					onClick={() => handleChange(null)}
					leftSection={isHidden(null) ? <IconSquare /> : <IconSquareCheck />}
				>
					Unknown
				</Button>
				{props.column.filterOptions.map((filterOption) => (
					<Button
						fullWidth
						variant={isHidden(filterOption.value) ? "light" : "filled"}
						key={filterOption.value}
						justify="start"
						leftSection={
							isHidden(filterOption.value) ? (
								<IconSquare />
							) : (
								<IconSquareCheck />
							)
						}
						onClick={() => {
							props.onChange(
								isHidden(filterOption.value)
									? selectedValues.filter((id) => id !== filterOption.value)
									: [...selectedValues, filterOption.value],
							);
						}}
					>
						{filterOption.label || filterOption.value}
					</Button>
				))}
			</Button.Group>
			<Button
				onClick={handleReset}
				leftSection={<IconRestore fontSize={10} />}
				size="compact-sm"
				disabled={(props.hiddenValues?.length || 0) === 0}
			>
				Reset
			</Button>
		</Stack>
	);
}
