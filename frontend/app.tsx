import { useData } from "@hooks/use-data";
import { AppNotifications } from "@components/app-notifications";
import { useAppUpdater } from "@hooks/use-app-updater";
import { AppTabs } from "@components/app-tabs";
import { commands } from "@api/bindings";

async function testDiscordOauth() {
	console.log("[Discord OAuth] Starting test flow...");

	try {
		const response = await commands.startDiscordOauth();
		console.log("[Discord OAuth] Success:", response);
	} catch (error) {
		console.error("[Discord OAuth] Failed:", error);
	}
}

function App() {
	useAppUpdater();
	useData();

	return (
		<>
			<div style={{ padding: 12 }}>
				<button
					type="button"
					onClick={testDiscordOauth}
				>
					Test Discord OAuth (Backend)
				</button>
			</div>
			<AppNotifications />
			<AppTabs />
		</>
	);
}

export default App;
