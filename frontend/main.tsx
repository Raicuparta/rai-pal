import React from "react";
import ReactDOM from "react-dom/client";
import { MantineProvider, v8CssVariablesResolver } from "@mantine/core";
import { commands } from "@api/bindings";
import App from "./app";
import { theme } from "./theme";
import { registerEvents } from "./register-events";
import { getVersion } from "@tauri-apps/api/app";

import "@mantine/core/styles.css";
import "@mantine/notifications/styles.css";
import "./global-styles/global.css";
import "./global-styles/mantine-overrides.css";
import "./global-styles/scroll-bar.css";
import { platform } from "@tauri-apps/plugin-os";

getVersion()
	.then((appVersion) => {
		commands.sendAnalyticsEvent("StartApp", {
			app_version: appVersion,
			platform: platform(),
			mode: import.meta.env.MODE,
		});
	})
	.catch((error) => {
		console.error(`Error trying to send analytics event: ${error}`);
	});

registerEvents();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
	<React.StrictMode>
		<MantineProvider
			defaultColorScheme="dark"
			cssVariablesResolver={v8CssVariablesResolver}
			theme={theme}
		>
			<App />
		</MantineProvider>
	</React.StrictMode>,
);
