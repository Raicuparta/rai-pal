import { CloseButton, Input } from "@mantine/core";

type Props = {
	readonly onChange: (search: string) => void;
	readonly value: string;
	readonly count: number;
};

export function SearchInput(props: Props) {
	return (
		<Input
			onChange={(event) => props.onChange(event.target.value)}
			placeholder={`Search (count: ${props.count})`}
			style={{ flex: 1 }}
			value={props.value}
			rightSectionPointerEvents="all"
			rightSection={
				<CloseButton
					aria-label="Reset search field"
					onClick={() => props.onChange("")}
					style={{ display: props.value ? undefined : "none" }}
				/>
			}
		/>
	);
}
