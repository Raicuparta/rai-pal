import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { ChakraProvider, DarkMode } from "@chakra-ui/react";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ChakraProvider>
      <DarkMode>
        <App />
      </DarkMode>
    </ChakraProvider>
  </React.StrictMode>
);
