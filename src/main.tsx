import React from "react";
import ReactDOM from "react-dom/client";
import { addCollection } from "@iconify/react";
import vscodeIcons from "@iconify-json/vscode-icons/icons.json";
import App from "./App";
import "antd/dist/reset.css";
import "./app/styles.css";

addCollection(vscodeIcons as never);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
