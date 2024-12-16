import { Table } from "@mantine/core";
import { TableColumn, TableHead } from "./table-head";
import React from "react";
import { TableContainer } from "./table-container";

type Props<TKey extends string, TData, TSort> = {
	readonly columns: TableColumn<TKey, TData, TSort>[];
	readonly item?: TData;
};

export function TableItemDetails<TKey extends string, TData, TSort>(
	props: Props<TKey, TData, TSort>,
) {
	const columns = props.columns.filter((column) => !column.hideInDetails);

	return (
		<TableContainer>
			<Table>
				<Table.Thead>
					<TableHead columns={columns} />
				</Table.Thead>
				<Table.Tbody>
					<Table.Tr>
						{columns.map(
							(column) =>
								!column.hideInDetails &&
								props.item && (
									<React.Fragment key={column.id}>
										<column.component item={props.item} />
									</React.Fragment>
								),
						)}
					</Table.Tr>
				</Table.Tbody>
			</Table>
		</TableContainer>
	);
}
