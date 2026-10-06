import { useCallback, useState } from "react";
import { useData } from "@hooks/use-data";
import { AppNotifications } from "@components/app-notifications";
import { useAppUpdater } from "@hooks/use-app-updater";
import { AppTabs } from "@components/app-tabs";
import { useAppEvent } from "@hooks/use-app-event";
import { ConfirmModSourceModal } from "@components/tools/confirm-mod-source-modal";
import { AppUpdateModal } from "@components/app-update-modal";

function App() {
	const { availableUpdate, installUpdate, ignoreUpdate } = useAppUpdater();
	useData();

	const [pendingSourceUrl, setPendingSourceUrl] = useState<string | null>(null);

	const handleAddModSource = useCallback((url: string) => {
		setPendingSourceUrl(url);
	}, []);

	useAppEvent("addModSource", "app", handleAddModSource);

	return (
		<>
			<AppNotifications />
			<AppTabs />
			<ConfirmModSourceModal
				url={pendingSourceUrl ?? ""}
				isOpen={!!pendingSourceUrl}
				onClose={() => setPendingSourceUrl(null)}
				onSaved={() => setPendingSourceUrl(null)}
			/>
			<AppUpdateModal
				update={availableUpdate}
				onInstall={installUpdate}
				onIgnore={ignoreUpdate}
			/>
		</>
	);
}

export default App;
