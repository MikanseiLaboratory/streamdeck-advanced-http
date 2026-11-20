import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { StreamDeckProvider } from "@mikanseilaboratory/streamdeck-pi-client";
import { App } from "./App";
import "./styles.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <StreamDeckProvider>
      <App />
    </StreamDeckProvider>
  </StrictMode>,
);
