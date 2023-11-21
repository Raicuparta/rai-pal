import { Box, Flex, Table } from "@mantine/core";
import classes from "./table.module.css";
import { IconChevronDown, IconChevronUp } from "@tabler/icons-react";
import { TableSort } from "@hooks/use-table-sort";
import { SegmentedControlData } from "@components/installed-games/typed-segmented-control";

export type TableColumn<TItem, TFilterOption extends string = string> = {
	id: string;
	label: string;
	renderCell: (item: TItem) => JSX.Element;
	width?: number;
	center?: boolean;
	hidable?: boolean;
	sort?: (itemA: TItem, itemB: TItem) => number;
	getSortValue?: (item: TItem) => unknown;
	filterOptions?: SegmentedControlData<TFilterOption>[];
};

type Props<TItem, TFilterOption extends string = string> = {
	readonly columns: TableColumn<TItem, TFilterOption>[];
	readonly onChangeSort?: (sort: string) => void;
	readonly sort?: TableSort;
};

export function TableHead<TItem, TFilterOption extends string = string>(
	props: Props<TItem, TFilterOption>,
) {
	return (
		<Table.Tr>
			{props.columns.map((column) => {
				const isSortable = Boolean(column.sort || column.getSortValue);
				return (
					<Table.Th
						className={
							props.onChangeSort && isSortable ? classes.sortable : undefined
						}
						key={String(column.id)}
						onClick={
							isSortable
								? () =>
										props.onChangeSort
											? props.onChangeSort(column.id)
											: undefined
								: undefined
						}
						w={column.width}
					>
						<Flex justify={column.center ? "center" : undefined}>
							{column.label}
							<Box
								h={0}
								w={0}
								fs="xs"
							>
								{props.sort?.id === column.id &&
									(props.sort.reverse ? (
										<IconChevronDown />
									) : (
										<IconChevronUp />
									))}
							</Box>
						</Flex>
					</Table.Th>
				);
			})}
		</Table.Tr>
	);
}
