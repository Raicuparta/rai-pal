import React from "react";
import ReactDOM from "react-dom/client";
import { MantineProvider } from "@mantine/core";
import App from "./app";
import { theme } from "./theme";
import { preventEvents } from "./prevent-events";

import "@mantine/core/styles.css";
import "./global-styles/global.css";
import "./global-styles/mantine-overrides.css";
import "./global-styles/scroll-bar.css";

preventEvents();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
	<React.StrictMode>
		<MantineProvider
			defaultColorScheme="dark"
			theme={theme}
		>
			<App />
		</MantineProvider>
	</React.StrictMode>,
);
