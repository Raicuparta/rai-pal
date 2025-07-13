import { Button, InputLabel, Stack } from "@mantine/core";
import { GamesFilter } from "@api/bindings";
import { IconRestore } from "@tabler/icons-react";
import { useLocalization } from "@hooks/use-localization";
import { CheckboxButton } from "@components/checkbox-button";
import { filterDetails } from "./filter-menu";

export type FilterKey = keyof GamesFilter;
export type FilterValue<TFilterKey extends FilterKey> =
	GamesFilter[TFilterKey][number];

export type FilterChangeCallback = (
	id: keyof GamesFilter,
	value: Array<FilterValue<typeof id>>,
) => void;

type Props<TFilterKey extends keyof GamesFilter> = {
	readonly id: TFilterKey;
	readonly possibleValues: Array<NonNullable<FilterValue<TFilterKey>>>;
	readonly currentValues: Array<FilterValue<TFilterKey>>;
	readonly onChange: FilterChangeCallback;
};

export function FilterSelect<TFilterKey extends FilterKey>({
	id,
	possibleValues,
	currentValues,
	onChange,
}: Props<TFilterKey>) {
	const tMenu = useLocalization("filterMenu");
	const tProperty = useLocalization("filterProperty");
	const tValue = useLocalization("filterValue");
	const tValueNote = useLocalization("filterValueNote");
	const emptyLocalizationKey = filterDetails[id].emptyLocalizationKey;
	const possibleValuesWithNull = [
		...(emptyLocalizationKey ? [null] : []),
		...possibleValues,
	];

	function handleFilterClick(id: TFilterKey, value: string | null) {
		let newValues = [
			...(currentValues.length == 0 ? possibleValuesWithNull : currentValues),
		];
		const index = newValues.indexOf(value as FilterValue<TFilterKey>);
		if (index === -1) {
			newValues.push(value as FilterValue<TFilterKey>);
		} else {
			newValues.splice(index, 1);
		}
		if (newValues.length == possibleValuesWithNull.length) {
			newValues = [];
		}

		onChange(id, newValues);
	}

	function handleResetClick() {
		onChange(id, []);
	}

	return (
		<Stack>
			<Stack
				gap={0}
				align="center"
			>
				<InputLabel>{tProperty(filterDetails[id].localizationKey)}</InputLabel>
				<Button.Group orientation="vertical">
					{possibleValues.map((possibleValue) => {
						const valueDetails = filterDetails[id].valueDetails[possibleValue];

						return (
							<CheckboxButton
								key={possibleValue}
								tooltip={tValueNote(valueDetails?.noteLocalizationKey)}
								checked={
									currentValues.length == 0 ||
									currentValues.includes(possibleValue)
								}
								onChange={() => handleFilterClick(id, possibleValue)}
							>
								{valueDetails?.staticDisplayText ??
									tValue(valueDetails?.localizationKey) ??
									possibleValue}
							</CheckboxButton>
						);
					})}
					{emptyLocalizationKey && (
						<CheckboxButton
							checked={
								currentValues.length == 0 || currentValues.includes(null)
							}
							onChange={() => handleFilterClick(id, null)}
						>
							{tValue(emptyLocalizationKey)}
						</CheckboxButton>
					)}
				</Button.Group>
			</Stack>
			<Button
				onClick={handleResetClick}
				leftSection={<IconRestore fontSize={10} />}
				disabled={currentValues.length == 0}
			>
				{tMenu("resetButton")}
			</Button>
		</Stack>
	);
}
