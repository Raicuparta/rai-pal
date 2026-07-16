import { useCallback, useState } from "react";
import { useData } from "@hooks/use-data";
import { AppNotifications } from "@components/app-notifications";
import { useAppUpdater } from "@hooks/use-app-updater";
import { AppTabs } from "@components/app-tabs";
import { useAppEvent } from "@hooks/use-app-event";
import { ConfirmModSourceModal } from "@components/tools/confirm-mod-source-modal";

function App() {
	useAppUpdater();
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
		</>
	);
}

export default App;
