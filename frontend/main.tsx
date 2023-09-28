import React from "react";
import ReactDOM from "react-dom/client";
import App from "./app";
import { MantineProvider } from "@mantine/core";
import { theme } from "./theme";

import "@mantine/core/styles.css";
import "./global-styles/global.css";
import "./global-styles/mantine-overrides.css";
import "./global-styles/scroll-bar.css";

// Prevent ctrl+p from opening the print dialog
document.addEventListener("keydown", (e) => {
  if (e.ctrlKey && e.key === "p") {
    e.preventDefault();
  }
});

// Prevent opening context menu
document.addEventListener("contextmenu", (e) => {
  e.preventDefault();
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <MantineProvider theme={theme} defaultColorScheme="dark">
      <App />
    </MantineProvider>
  </React.StrictMode>
);
