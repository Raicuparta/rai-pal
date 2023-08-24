import React from "react";
import ReactDOM from "react-dom/client";
import App from "./app";
import { ChakraProvider, DarkMode } from "@chakra-ui/react";
import "./scroll-bars.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ChakraProvider>
      <DarkMode>
        <App />
      </DarkMode>
    </ChakraProvider>
  </React.StrictMode>
);
