import { useLocalization } from "@hooks/use-localization";
import { CloseButton, Input } from "@mantine/core";

type Props = {
	readonly onChange: (search: string) => void;
	readonly value: string;
};

export function SearchInput(props: Props) {
	const t = useLocalization("filterMenu");

	// TODO on linux it seems to be losing key presses whe typing fast
	return (
		<Input
			onChange={(event) => props.onChange(event.target.value)}
			placeholder={t("searchPlaceholder")}
			style={{ flex: 1 }}
			value={props.value}
			rightSectionPointerEvents="all"
			rightSection={
				<CloseButton
					onClick={() => props.onChange("")}
					style={{ display: props.value ? undefined : "none" }}
				/>
			}
		/>
	);
}
