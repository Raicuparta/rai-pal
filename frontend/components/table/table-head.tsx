import { Box, Flex } from "@mantine/core";
import { IconChevronDown, IconChevronUp } from "@tabler/icons-react";
import { GamesSortBy } from "@api/bindings";
import { css } from "@styled-system/css";

export type TableColumnBase<TItem, TSort> = {
	label: string;
	component: React.ComponentType<{ item: TItem }>;
	width?: number;
	center?: boolean;
	hideLabel?: boolean;
	hidable?: boolean;
	sort?: TSort;
};

export interface TableColumn<TKey extends string, TItem, TSort>
	extends TableColumnBase<TItem, TSort> {
	id: TKey;
}

export function columnMapToList<TItem, TKey extends string, TSort>(
	columnMap: Record<TKey, TableColumnBase<TItem, TSort>>,
): TableColumn<TKey, TItem, TSort>[] {
	return Object.entries<TableColumnBase<TItem, TSort>>(columnMap).map(
		([id, column]) => ({
			...column,
			id: id as TKey,
		}),
	);
}

type Props<TKey extends string, TItem, TSort> = {
	readonly columns: TableColumn<TKey, TItem, TSort>[];
	readonly onChangeSort?: (sortBy: TSort) => void;
	readonly sortBy?: GamesSortBy;
	readonly sortDescending?: boolean;
};

export function TableHead<TKey extends string, TItem, TSort>(
	props: Props<TKey, TItem, TSort>,
) {
	return (
		<tr>
			{props.columns.map((column) => {
				return (
					<th
						key={String(column.id)}
						className={css({
							cursor: column.sort && props.onChangeSort && "pointer",
							_hover: column.sort &&
								props.onChangeSort && {
									background:
										"light-dark(var(--mantine-color-gray-3), var(--mantine-color-dark-7))",
								},
							padding: 1,
						})}
						style={{
							width: column.width,
						}}
						onClick={
							column.sort
								? () =>
										props.onChangeSort && column.sort
											? props.onChangeSort(column.sort)
											: undefined
								: undefined
						}
					>
						<Flex justify={column.center ? "center" : undefined}>
							{column.hideLabel ? "" : column.label}
							<Box
								h={0}
								w={0}
								fs="xs"
							>
								{props.sortBy &&
									props.sortBy === column.sort &&
									(props.sortDescending ? (
										<IconChevronDown />
									) : (
										<IconChevronUp />
									))}
							</Box>
						</Flex>
					</th>
				);
			})}
		</tr>
	);
}
