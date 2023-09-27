import React from "react";
import ReactDOM from "react-dom/client";
import App from "./app";
import { MantineProvider } from "@mantine/core";
import { theme } from "./theme";

import "@mantine/core/styles.css";
import "./global.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <MantineProvider theme={theme} defaultColorScheme="dark">
      <App />
    </MantineProvider>
  </React.StrictMode>
);
