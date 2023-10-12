import { Box, Flex, Table } from "@mantine/core";
import classes from "./table.module.css";
import { IconChevronDown, IconChevronUp } from "@tabler/icons-react";
import { TableSort } from "@hooks/use-table-sort";

export type TableHeader<TItem> = {
	id: string;
	label: string;
	width?: number;
	center?: boolean;
	sort?: (itemA: TItem, itemB: TItem) => number;
	getSortValue?: (item: TItem) => unknown;
};

type Props<TItem> = {
	readonly headers: TableHeader<TItem>[];
	readonly onChangeSort?: (sort: string) => void;
	readonly sort?: TableSort;
};

export function TableHead<TItem>(props: Props<TItem>) {
	return (
		<Table.Tr>
			{props.headers.map((header) => {
				const isSortable = Boolean(header.sort || header.getSortValue);
				return (
					<Table.Th
						className={
							props.onChangeSort && isSortable ? classes.sortable : undefined
						}
						key={String(header.id)}
						onClick={
							isSortable
								? () =>
										props.onChangeSort
											? props.onChangeSort(header.id)
											: undefined
								: undefined
						}
						w={header.width}
					>
						<Flex justify={header.center ? "center" : undefined}>
							{header.label}
							<Box
								h={0}
								w={0}
								fs="xs"
							>
								{props.sort?.id === header.id &&
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
