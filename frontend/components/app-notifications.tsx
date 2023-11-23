import { useAppEvent } from "@hooks/use-app-event";
import { Notifications, notifications } from "@mantine/notifications";

export function AppNotifications() {
	useAppEvent("ExecutedSteamCommand", () => {
		notifications.show({
			message: "Running steam command...",
			color: "blue",
		});
	});

	useAppEvent<string>("GameAdded", (gameName) => {
		notifications.show({
			message: `Successfully added game "${gameName}".`,
			color: "green",
		});
	});

	useAppEvent<string>("GameRemoved", (gameName) => {
		notifications.show({
			message: `Removed game "${gameName}".`,
			color: "green",
		});
	});

	return <Notifications />;
}
