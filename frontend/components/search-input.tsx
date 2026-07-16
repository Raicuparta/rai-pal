import { useLocalization } from "@hooks/use-localization";
import { CloseButton, Input } from "@mantine/core";
import { useDebouncedCallback } from "@mantine/hooks";
import { useState } from "react";

type Props = {
	readonly onChange: (search: string) => void;
	readonly value: string;
};

export function SearchInput(props: Props) {
	const t = useLocalization("filterMenu");

	const [prevValueProp, setPrevValueProp] = useState(props.value);
	const [innerValue, setInnerValue] = useState(props.value);

	if (props.value !== prevValueProp) {
		setPrevValueProp(props.value);
		setInnerValue(props.value);
	}

	const debouncedOnChange = useDebouncedCallback(props.onChange, 200);

	const setValue = (value: string) => {
		setInnerValue(value);
		debouncedOnChange(value);
	};

	return (
		<Input
			onChange={(event) => {
				setValue(event.currentTarget.value);
			}}
			placeholder={t("searchPlaceholder")}
			style={{ flex: 1 }}
			value={innerValue}
			rightSectionPointerEvents="all"
			rightSection={
				<CloseButton
					onClick={() => setValue("")}
					style={{ display: innerValue ? undefined : "none" }}
				/>
			}
		/>
	);
}
