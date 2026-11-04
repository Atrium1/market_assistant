import React from "react";
import ReactDOM from "react-dom/client";
import { HashRouter } from "react-router-dom";
import { OwnerSessionProvider } from "./auth/OwnerSessionContext";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <HashRouter>
      <OwnerSessionProvider>
        <App />
      </OwnerSessionProvider>
    </HashRouter>
  </React.StrictMode>,
);
