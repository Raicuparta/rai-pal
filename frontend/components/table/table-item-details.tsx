import { Table } from "@mantine/core";
import { TableColumn, TableHead } from "./table-head";
import React from "react";
import { TableContainer } from "./table-container";

type Props<TKey extends string, TData> = {
	readonly columns: TableColumn<TKey, TData>[];
	readonly item: TData;
};

export function TableItemDetails<TKey extends string, TData>(
	props: Props<TKey, TData>,
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
								!column.hideInDetails && (
									<React.Fragment key={column.id}>
										{column.renderCell(props.item)}
									</React.Fragment>
								),
						)}
					</Table.Tr>
				</Table.Tbody>
			</Table>
		</TableContainer>
	);
}
