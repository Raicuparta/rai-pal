import { Box } from "@mantine/core";
import { CommandButton } from "@components/command-button";
import { ComponentProps, forwardRef } from "react";

type Props = ComponentProps<typeof CommandButton>;

function GameModActionButtonInternal({ children, ...props }: Props) {
	return (
		<CommandButton
			size="xs"
			{...props}
		>
			<Box style={{ textOverflow: "ellipsis", overflow: "hidden" }}>
				{children}
			</Box>
		</CommandButton>
	);
}

export const GameModActionButton = forwardRef(GameModActionButtonInternal);
