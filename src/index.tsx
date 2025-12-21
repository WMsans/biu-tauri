import { createRoot } from "react-dom/client";
import { HashRouter } from "react-router";

import { App } from "./app";

import "./i18n";

const root = createRoot(document.getElementById("root") as Element);
root.render(
  <HashRouter>
    <App />
  </HashRouter>,
);
