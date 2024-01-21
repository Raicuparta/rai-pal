import { SegmentedControl, SegmentedControlItem, Tooltip } from "@mantine/core";

export type SegmentedControlData<T extends string> = SegmentedControlItem & {
	value: T | "";
};

export type TypedSegmentedControlProps<T extends string> = {
	readonly data: SegmentedControlData<T>[];
	readonly onChange: (value?: T) => void;
	readonly value?: T;
	readonly unavailableValues?: T[];
};

export function TypedSegmentedControl<T extends string>(
	props: TypedSegmentedControlProps<T>,
) {
	return (
		<SegmentedControl
			my={-2.5}
			color={props.value ? "violet" : undefined}
			data={props.data.map((data) => {
				const isUnavailable =
					!!data.value && props.unavailableValues?.includes(data.value);

				return {
					...data,
					disabled: isUnavailable,
					label: (
						<Tooltip
							label="Not implemented"
							disabled={!isUnavailable}
						>
							<div>{data.label || data.value}</div>
						</Tooltip>
					),
				};
			})}
			onChange={(value) => props.onChange((value as T) || undefined)}
			value={props.value || ""}
		/>
	);
}
