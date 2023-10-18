import { useAppEvent } from "@hooks/use-app-event";
import { Notifications, notifications } from "@mantine/notifications";

export function AppNotifications() {
	useAppEvent("ExecutedSteamCommand", () => {
		notifications.show({
			message: "Running steam command...",
		});
	});

	return <Notifications />;
}
