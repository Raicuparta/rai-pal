import { Box, Flex, Table } from "@mantine/core";
import classes from "./table.module.css";
import { IconChevronDown, IconChevronUp } from "@tabler/icons-react";
import { TableSort } from "@hooks/use-table-sort";
import { SegmentedControlData } from "@components/installed-games/typed-segmented-control";

export type TableColumnBase<TItem, TFilterOption extends string = string> = {
	label: string;
	renderCell: (item: TItem) => JSX.Element;
	width?: number;
	center?: boolean;
	hideLabel?: boolean;
	hidable?: boolean;
	hideInDetails?: boolean;
	sort?: (itemA: TItem, itemB: TItem) => number;
	getSortValue?: (item: TItem) => unknown;
	getFilterValue?: (item: TItem) => TFilterOption | "";
	filterOptions?: SegmentedControlData<TFilterOption>[];
};

export interface TableColumn<
	TKey extends string,
	TItem,
	TFilterOption extends string = string,
> extends TableColumnBase<TItem, TFilterOption> {
	id: TKey;
}

export function columnMapToList<
	TItem,
	TKey extends string,
	TFilterOption extends string = string,
>(
	columnMap: Record<TKey, TableColumnBase<TItem, TFilterOption>>,
): TableColumn<TKey, TItem, TFilterOption>[] {
	return Object.entries<TableColumnBase<TItem, TFilterOption>>(columnMap).map(
		([id, column]) => ({
			...column,
			id: id as TKey,
		}),
	);
}

type Props<
	TKey extends string,
	TItem,
	TFilterOption extends string = string,
> = {
	readonly columns: TableColumn<TKey, TItem, TFilterOption>[];
	readonly onChangeSort?: (sort: string) => void;
	readonly sort?: TableSort;
};

export function TableHead<
	TKey extends string,
	TItem,
	TFilterOption extends string = string,
>(props: Props<TKey, TItem, TFilterOption>) {
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
							{column.hideLabel ? "" : column.label}
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
