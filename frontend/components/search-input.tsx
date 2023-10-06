import { Filter } from "@hooks/use-filtered-list";
import { CloseButton, Input } from "@mantine/core";

type Props = {
	readonly onChange: (filter: Filter) => void;
	readonly value: string;
	readonly count: number;
};

export function SearchInput(props: Props) {
	return (
		<Input
			onChange={(event) => props.onChange({ search: event.target.value })}
			placeholder={`Search ${props.count} games...`}
			style={{ flex: 1 }}
			value={props.value}
			rightSectionPointerEvents="all"
			rightSection={
				<CloseButton
					aria-label="Reset search field"
					onClick={() => props.onChange({ search: "" })}
					style={{ display: props.value ? undefined : "none" }}
				/>
			}
		/>
	);
}
