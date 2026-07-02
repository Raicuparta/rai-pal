import { ActionIcon, Button, ButtonProps, Group, Tooltip } from "@mantine/core";
import { forwardRef } from "react";
import { IconCheck } from "@tabler/icons-react";

interface Props extends ButtonProps {
	readonly checked: boolean;
	readonly onClickCheckbox: () => void;
	readonly onClickButton: () => void;
	readonly tooltip?: string;
}

function CheckboxButtonInternal(
	{
		checked,
		onClickCheckbox,
		onClickButton,
		tooltip,
		children,
		...props
	}: Props,
	ref: React.ForwardedRef<HTMLButtonElement>,
) {
	return (
		<Tooltip
			label={tooltip}
			disabled={!tooltip}
		>
			<Group gap={0}>
				<ActionIcon
					size="sm"
					variant={checked ? "filled" : "default"}
					onClick={onClickCheckbox}
				>
					{checked && <IconCheck />}
				</ActionIcon>
				<Button
					variant="subtle"
					color="gray"
					ref={ref}
					justify="start"
					flex={1}
					px="xs"
					onClick={(e) => {
						e.stopPropagation();
						onClickButton();
					}}
					{...props}
				>
					{children}

					{tooltip && " *"}
				</Button>
			</Group>
		</Tooltip>
	);
}

export const CheckboxButton = forwardRef(CheckboxButtonInternal);
