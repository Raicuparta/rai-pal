import { Button, Stack, Tooltip } from "@mantine/core";
import { resetLocalStorage } from "../../util/local-storage";
import { useEffect, useState } from "react";
import { getOtherGames } from "@api/bindings";
import { DebugData } from "@components/debug-data";

export function SettingsPage() {
	const [otherGames, setOtherGames] = useState<unknown>();

	useEffect(() => {
		getOtherGames().then(setOtherGames);
	}, []);

	return (
		<Stack>
			<Tooltip
				label="Will reset filters, confirmation dialogs, probably other stuff."
				position="bottom"
			>
				<Button onClick={resetLocalStorage}>Reset settings to defaults</Button>
			</Tooltip>
			<DebugData data={otherGames} />
		</Stack>
	);
}
