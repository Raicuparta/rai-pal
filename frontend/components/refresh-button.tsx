import { useLongLoading } from "@hooks/use-long-loading";
import { Button } from "@mantine/core";
import { IconRefresh } from "@tabler/icons-react";
import React from "react";

type Props = {
	readonly onClick: () => void;
	readonly loading: boolean;
};

function RefreshButtonInner(
	props: Props,
	ref: React.ForwardedRef<HTMLButtonElement>,
) {
	const isLoading = useLongLoading(props.loading);

	return (
		<Button
			ref={ref}
			leftSection={<IconRefresh />}
			loading={isLoading}
			onClick={props.onClick}
			style={{ flex: 1, maxWidth: "10em" }}
			variant="filled"
		>
			Refresh
		</Button>
	);
}

export const RefreshButton = React.forwardRef(RefreshButtonInner);
