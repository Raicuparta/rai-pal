import { Table } from "@mantine/core";
import { TableColumn, TableHead } from "./table-head";
import React from "react";
import { TableContainer } from "./table-container";

type Props<TData> = {
	readonly columns: TableColumn<TData>[];
	readonly item: TData;
};

export function TableItemDetails<TData>(props: Props<TData>) {
	const columns = props.columns.filter((column) => !column.hideInDetails);

	return (
		<TableContainer>
			<Table>
				<thead>
					<TableHead columns={columns} />
				</thead>
				<tbody>
					<tr>
						{columns.map(
							(column) =>
								!column.hideInDetails && (
									<React.Fragment key={column.id}>
										{column.renderCell(props.item)}
									</React.Fragment>
								),
						)}
					</tr>
				</tbody>
			</Table>
		</TableContainer>
	);
}
