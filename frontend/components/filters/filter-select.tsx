import { Button, Stack } from "@mantine/core";
import { FilterButton } from "./filter-button";
import { GamesFilter } from "@api/bindings";
import { defaultQuery, useDataQuery } from "@hooks/use-data-query";

type Props = {
	readonly id: keyof GamesFilter;
	readonly onClick: (
		id: keyof GamesFilter,
		value: GamesFilter[typeof id][number],
	) => void;
};

export function FilterSelect({ id, onClick }: Props) {
	const [dataQuery] = useDataQuery();

	return (
		<Stack gap="xs">
			<Button.Group orientation="vertical">
				<Button disabled>{id}</Button>
				{defaultQuery.filter[id].map((option) => (
					<FilterButton
						filterOption={option ?? "Unknown"}
						onClick={() => onClick(id, option)}
						isVisible={dataQuery.filter[id].includes(option)}
						// isUnavailable={Boolean(
						// 	filterOption &&
						// 		column.unavailableValues?.includes(filterOption),
						// )}
						isUnavailable={false} // TODO isUnavailable
						key={option}
					/>
				))}
			</Button.Group>
			{/* <Button
						onClick={handleReset}
						leftSection={<IconRestore fontSize={10} />}
						disabled={(hiddenValues?.length || 0) === 0}
					>
						Reset
					</Button> */}
		</Stack>
	);
}
