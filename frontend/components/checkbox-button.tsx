import { Button, ButtonProps, Tooltip } from "@mantine/core";
import { forwardRef } from "react";
import {
	IconCheck,
	IconLock,
	IconLockOpen,
	IconMinus,
} from "@tabler/icons-react";

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
		<Button.Group flex={1}>
			<Button
				size="compact-xs"
				px={2}
				variant="subtle"
				onClick={onClickCheckbox}
				disabled={locked}
				opacity={locked ? 0.4 : 1}
				c={checked ? "violet" : "bright"}
			>
				{checked ? (
					<IconCheck
						stroke="4px"
						color="currentcolor"
					/>
				) : (
					<IconMinus />
				)}
			</Button>
			<Tooltip
				label={tooltip}
				disabled={!tooltip}
			>
				<Button
					disabled={locked}
					variant="subtle"
					size="compact-xs"
					ref={ref}
					justify="start"
					opacity={locked ? 0.4 : 1}
					c="bright"
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
			</Tooltip>
			{onClickLock && (
				<Button
					px={2}
					size="compact-xs"
					variant="light"
					bg={locked ? undefined : "transparent"}
					color={locked ? "yellow" : "gray"}
					disabled={checked}
					onClick={onClickLock}
				>
					{locked ? <IconLock /> : <IconLockOpen />}
				</Button>
			)}
		</Button.Group>
	);
}

export const CheckboxButton = forwardRef(CheckboxButtonInternal);
