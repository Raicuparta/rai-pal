import { Button, Stack } from "@mantine/core";
import { FilterButton } from "./filter-button";
import { GamesFilter } from "@api/bindings";

export type FilterKey = keyof GamesFilter;
export type FilterValue<TFilterKey extends FilterKey> =
	GamesFilter[TFilterKey][number];

type Props<TFilterKey extends keyof GamesFilter> = {
	readonly id: TFilterKey;
	readonly possibleValues: Array<FilterValue<TFilterKey>>;
	readonly currentValues: Array<FilterValue<TFilterKey>>;
	readonly onClick: (id: TFilterKey, value: FilterValue<TFilterKey>) => void;
};

export function FilterSelect<TFilterKey extends FilterKey>({
	id,
	possibleValues,
	currentValues,
	onClick,
}: Props<TFilterKey>) {
	return (
		<Stack gap="xs">
			<Button.Group orientation="vertical">
				<Button disabled>{id}</Button>
				{possibleValues.map((possibleValue) => (
					<FilterButton
						filterOption={possibleValue ?? "Unknown"}
						onClick={() => onClick(id, possibleValue)}
						isVisible={currentValues.includes(possibleValue)}
						// isUnavailable={Boolean(
						// 	filterOption &&
						// 		column.unavailableValues?.includes(filterOption),
						// )}
						isUnavailable={false} // TODO isUnavailable
						key={possibleValue}
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
