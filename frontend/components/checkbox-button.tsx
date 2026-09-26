import { Button, ButtonProps, Tooltip } from "@mantine/core";
import { forwardRef } from "react";
import { IconCheck, IconLock, IconLockOpen, IconX } from "@tabler/icons-react";

interface Props extends ButtonProps {
	readonly checked: boolean;
	readonly locked?: boolean;
	readonly onClickCheckbox: () => void;
	readonly onClickButton: () => void;
	readonly onClickLock?: () => void;
	readonly tooltip?: string;
}

function CheckboxButtonInternal(
	{
		checked,
		locked = false,
		onClickCheckbox,
		onClickButton,
		onClickLock,
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
			<Button.Group flex={1}>
				{onClickLock && (
					<Button
						px={4}
						size="compact-xs"
						variant="light"
						bg={locked ? undefined : "transparent"}
						color={locked ? "yellow" : "white"}
						disabled={checked}
						onClick={onClickLock}
					>
						{locked ? <IconLock /> : <IconLockOpen />}
					</Button>
				)}
				<Button
					size="compact-xs"
					px={4}
					variant={checked ? "filled" : "light"}
					onClick={onClickCheckbox}
				>
					{checked ? <IconCheck /> : <IconX opacity={0.3} />}
				</Button>
				<Button
					variant="light"
					size="compact-xs"
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
			</Button.Group>
		</Tooltip>
	);
}

export const CheckboxButton = forwardRef(CheckboxButtonInternal);
