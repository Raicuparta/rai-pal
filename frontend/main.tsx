import React from "react";
import ReactDOM from "react-dom/client";
import App from "./app";
import { MantineProvider } from "@mantine/core";
import { theme } from "./theme";

import "@mantine/core/styles.css";
import "./global.css";

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
