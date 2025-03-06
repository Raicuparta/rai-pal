import { Box, Flex, Table } from "@mantine/core";
import classes from "./table.module.css";
import { IconChevronDown, IconChevronUp } from "@tabler/icons-react";
import { GamesSortBy } from "@api/bindings";
import { TranslationKey, useGetTranslated } from "@hooks/use-translations";
import { useAppSettings } from "@hooks/use-app-settings";

export type TableColumnBase<TItem, TSort> = {
	translationKey?: TranslationKey<"gamesTableColumn">;
	component: React.ComponentType<{ item: TItem }>;
	width?: number;
	center?: boolean;
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
	const t = useGetTranslated("gamesTableColumn");
	const [settings] = useAppSettings();

	return (
		<Table.Tr>
			{props.columns.map((column) => {
				return (!settings.hideGameThumbnails || column.id !== "thumbnail") && (
					<Table.Th
						className={
							props.onChangeSort && column.sort ? classes.sortable : undefined
						}
						key={String(column.id)}
						onClick={
							column.sort
								? () =>
										props.onChangeSort && column.sort
											? props.onChangeSort(column.sort)
											: undefined
								: undefined
						}
						w={column.width}
						maw={column.width}
						miw={column.width}
					>
						<Flex justify={column.center ? "center" : undefined}>
							{t(column.translationKey)}
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
					</Table.Th>
				);
			})}
		</Table.Tr>
	);
}
