import { Button, InputLabel } from "@mantine/core";
import { TableColumn } from "./table/table-head";

type Props<
	TKey extends string,
	TItem,
	TFilterOption extends string = string,
> = {
	readonly columns: TableColumn<TKey, TItem, TFilterOption>[];
	readonly hiddenIds: TKey[];
	readonly onChange: (hiddenIds: TKey[]) => void;
};

export function ColumnsSelect<TKey extends string, TItem>(
	props: Props<TKey, TItem>,
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
									? "default"
									: "light"
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
