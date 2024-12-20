import { Button, InputLabel } from "@mantine/core";
import { TableColumn } from "./table/table-head";

type Props<TKey extends string, TItem, TSort> = {
	readonly columns: TableColumn<TKey, TItem, TSort>[];
	readonly hiddenIds: TKey[];
	readonly onChange: (hiddenIds: TKey[]) => void;
};

export function ColumnsSelect<TKey extends string, TItem, TSort>(
	props: Props<TKey, TItem, TSort>,
) {
	return (
		<div>
			<InputLabel>Table columns:</InputLabel>
			<Button.Group>
				{props.columns
					.filter(({ hidable }) => hidable)
					.map((column) => (
						<Button
							variant={
								props.hiddenIds.find((id) => id === column.id)
									? "filled"
									: "default"
							}
							key={column.id}
							onClick={() => {
								props.onChange(
									props.hiddenIds.find((id) => id === column.id)
										? props.hiddenIds.filter((id) => id !== column.id)
										: [...props.hiddenIds, column.id],
								);
							}}
						>
							{column.label || column.id}
						</Button>
					))}
			</Button.Group>
		</div>
	);
}
