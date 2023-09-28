import { Box, Flex, Table } from "@mantine/core";
import classes from "./table.module.css";
import { IconChevronDown, IconChevronUp } from "@tabler/icons-react";

export type TableHeader<TItem, TKey extends keyof TItem> = {
	id: TKey;
	label: string;
	width?: number;
	center?: boolean;
	customSort?: (itemA: TItem, itemB: TItem) => number;
};

type TableSort<TItem, TKey extends keyof TItem> = {
	id: TKey;
	reverse: boolean;
};

type Props<TItem, TKey extends keyof TItem> = {
	readonly headers: TableHeader<TItem, TKey>[];
	readonly onChangeSort?: (sort: TKey) => void;
	readonly sort?: TableSort<TItem, TKey>;
};

export function TableHead<TItem, TKey extends keyof TItem>(
	props: Props<TItem, TKey>,
) {
	return (
		<Table.Tr>
			{props.headers.map((header) => (
				<Table.Th
					className={props.onChangeSort ? classes.sortable : undefined}
					key={String(header.id)}
					onClick={() =>
						props.onChangeSort ? props.onChangeSort(header.id) : undefined
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
								(props.sort.reverse ? <IconChevronDown /> : <IconChevronUp />)}
						</Box>
					</Flex>
				</Table.Th>
			))}
		</Table.Tr>
	);
}
