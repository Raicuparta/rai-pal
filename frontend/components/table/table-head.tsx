import { Box, Flex, Table } from "@mantine/core";
import classes from "./table.module.css";
import { IconChevronDown, IconChevronUp } from "@tabler/icons-react";
import { TableSort } from "@hooks/use-table-sort";

export type TableColumnBase<TItem> = {
	label: string;
	renderCell: (item: TItem) => JSX.Element;
	width?: number;
	center?: boolean;
	hideLabel?: boolean;
	hidable?: boolean;
	hideInDetails?: boolean;
};

export interface TableColumn<TKey extends string, TItem>
	extends TableColumnBase<TItem> {
	id: TKey;
}

export function columnMapToList<TItem, TKey extends string>(
	columnMap: Record<TKey, TableColumnBase<TItem>>,
): TableColumn<TKey, TItem>[] {
	return Object.entries<TableColumnBase<TItem>>(columnMap).map(
		([id, column]) => ({
			...column,
			id: id as TKey,
		}),
	);
}

type Props<TKey extends string, TItem> = {
	readonly columns: TableColumn<TKey, TItem>[];
	readonly onChangeSort?: (sort: string) => void;
	readonly sort?: TableSort;
};

export function TableHead<TKey extends string, TItem>(
	props: Props<TKey, TItem>,
) {
	return (
		<Table.Tr>
			{props.columns.map((column) => {
				const isSortable = true; // TODO.
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
						maw={column.width}
						miw={column.width}
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
