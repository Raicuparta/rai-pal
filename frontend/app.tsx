import { useData } from "@hooks/use-data";
import { AppNotifications } from "@components/app-notifications";
import { useAppUpdater } from "@hooks/use-app-updater";
import { AppTabs } from "@components/app-tabs";

function App() {
	useAppUpdater();
	useData();

	return (
		<>
			<AppNotifications />
			<AppTabs />
		</>
	);
}

export default App;
