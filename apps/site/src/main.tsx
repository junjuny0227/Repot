import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import { isLocale } from "./i18n";
import "./styles/site.css";

// Each locale has its own static page; its <html lang> says which messages to use.
const lang = document.documentElement.lang;
createRoot(document.getElementById("root")!).render(<StrictMode><App locale={isLocale(lang) ? lang : "en"} /></StrictMode>);
