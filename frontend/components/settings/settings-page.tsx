import { Button, Stack, Tooltip } from "@mantine/core";
import { resetLocalStorage } from "../../util/local-storage";

export function SettingsPage() {
	return (
		<Stack>
			<Tooltip
				label="Will reset filters, confirmation dialogs, probably other stuff."
				position="bottom"
			>
				<Button onClick={resetLocalStorage}>Reset settings to defaults</Button>
			</Tooltip>
		</Stack>
	);
}
